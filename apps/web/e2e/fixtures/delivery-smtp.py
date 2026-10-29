"""Loopback-only SMTP acceptance fixture, including the RFC 5321 null sender.

No delivery or outbound connection. Advertises DSN and records envelope options.
A local drop-final marker simulates acceptance with its reply lost.
Usage: python delivery-smtp.py PORT OUTPUT_DIRECTORY

With MW_CAPTURE_RELAY=host:port set, each accepted message is also handed to that
SMTP server (a loopback Greenmail in the receipts stack), with the same envelope,
so specs that read the recipient's mailbox see it arrive. The relay is the only
outbound connection and it is off by default.
"""
import asyncio
import base64
import hashlib
import json
import os
import pathlib
import smtplib
import sys
import time

output = pathlib.Path(sys.argv[2]).resolve()
output.mkdir(parents=True, exist_ok=True)
slots = asyncio.Semaphore(8)
relay = os.environ.get('MW_CAPTURE_RELAY', '')


def forward(sender, recipients, raw):
    host, port = relay.rsplit(':', 1)
    with smtplib.SMTP(host, int(port), timeout=15) as client:
        client.sendmail(sender, recipients, bytes(raw))


async def session(reader, writer):
    async with slots:
        async def reply(value):
            writer.write((value + '\r\n').encode())
            await writer.drain()

        authenticated = False
        sender = None  # None means MAIL absent; empty string is valid MAIL FROM:<>.
        recipients = []
        envelope_commands = []
        try:
            await reply('220 delivery-fixture ESMTP')
            for _ in range(1000):
                line = await asyncio.wait_for(reader.readline(), 15)
                if not line:
                    return
                command = line.decode('ascii', 'strict').rstrip('\r\n')
                upper = command.upper()
                if upper.startswith('EHLO '):
                    await reply('250-delivery-fixture\r\n250-AUTH PLAIN\r\n250-DSN\r\n250 8BITMIME')
                elif upper.startswith('AUTH PLAIN '):
                    authenticated = base64.b64decode(command[11:], validate=True) == b'\x00testuser\x00testpass'
                    await reply('235 authenticated' if authenticated else '535 refused')
                elif upper.startswith('MAIL FROM:<') and authenticated:
                    envelope_commands = [command]
                    sender = command[11:].split('>', 1)[0]
                    recipients = []
                    await reply('250 sender accepted')
                elif upper.startswith('RCPT TO:<') and sender is not None:
                    envelope_commands.append(command)
                    recipients.append(command[9:].split('>', 1)[0])
                    await reply('250 recipient accepted')
                elif upper == 'DATA' and sender is not None and recipients:
                    await reply('354 finish with dot')
                    raw = bytearray()
                    while True:
                        part = await asyncio.wait_for(reader.readline(), 15)
                        if part == b'.\r\n':
                            break
                        if not part or len(raw) + len(part) > 1024 * 1024:
                            return
                        raw.extend(part[1:] if part.startswith(b'..') else part)
                    digest = hashlib.sha256(raw).hexdigest()
                    (output / f'{digest}.eml').write_bytes(raw)
                    with (output / 'accepted.jsonl').open('a', encoding='utf8') as log:
                        log.write(json.dumps({'sender': sender, 'recipients': recipients, 'sha256': digest, 'time': time.time(), 'commands': envelope_commands, 'replyDropped': (output / 'drop-final').exists()}) + '\n')
                    if relay:
                        try:
                            await asyncio.to_thread(forward, sender, list(recipients), raw)
                        except (OSError, smtplib.SMTPException) as error:
                            print(f'relay failed: {error}', file=sys.stderr)
                    if (output / 'drop-final').exists():
                        return
                    await reply('250 accepted')
                    sender = None
                    recipients = []
                elif upper == 'RSET':
                    sender = None
                    recipients = []
                    await reply('250 reset')
                elif upper == 'QUIT':
                    await reply('221 bye')
                    return
                else:
                    await reply('503 sequence refused')
        except (TimeoutError, ValueError, UnicodeError, ConnectionError):
            pass
        finally:
            writer.close()
            await writer.wait_closed()


async def main():
    server = await asyncio.start_server(session, '127.0.0.1', int(sys.argv[1]), limit=16 * 1024)
    async with server:
        await server.serve_forever()


asyncio.run(main())
