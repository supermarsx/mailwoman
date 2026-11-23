---
screen: true
---
# SSH tunnels

An SSH tunnel lets the server reach a hosting panel through an SSH server (a jump host) instead of connecting to it directly. A saved tunnel opens only SSH forwarding channels (`direct-tcpip`) to the destinations you list; it never runs commands or opens a shell. Tunnels are used by [hosting panel connections](./hosting.md); mail connections and DNS lookups do not go through them.

![The SSH tunnels tab of Integrations with no tunnels configured, and the New SSH tunnel and Refresh buttons.](/screenshots/admin-access/ssh-tunnels-list.webp){.mw-shot}

## Before you start

- The tab is **Integrations** > **SSH tunnels**, at `<console path>#/integrations/ssh`. It appears only to roles with `tunnels.read`.
- Creating, changing and deleting a tunnel needs `tunnels.write` for all domains (unscoped); inspecting a host key and testing need `tunnels.test`; choosing a tunnel for a hosting connection needs `tunnels.bind`. Changes, tests and inspections also ask you to confirm it is you (**Confirm it is you**).
- Have the SSH server's host-key fingerprint from a source you trust (for example the server's console output), so you can check the key the console shows.
- A tunnel cannot be used while an outbound proxy is active for the whole server: the connection fails with "SSH through an active global proxy is not configured" and never falls back to a direct connection.

## Create a tunnel

1. Choose **New SSH tunnel**.

2. Under **SSH server identity**, enter **Tunnel name**, **SSH server hostname** (a name or IP address, without `ssh://`, user name, path or port), **SSH server port** (22 by default) and **SSH login username**. Tick **Allow the SSH server on a private or loopback address** only if the jump host is on your internal network.

   ![The SSH server identity section of a new SSH tunnel: Tunnel name, SSH server hostname, SSH server port 22, SSH login username, Enable this tunnel and the private-address checkbox.](/screenshots/admin-access/ssh-tunnels-editor.webp){.mw-crop}

3. Under **Authentication credential**, choose what **Saved credential** does: generate an Ed25519 key, import a private key (PEM or OpenSSH, with an optional **Private-key passphrase (optional)**), or use an **SSH password**. For a generated key, install the shown **Public key to install in authorized_keys** on the SSH server. Imported keys must be Ed25519, ECDSA P-256 or P-384, or RSA of at least 3072 bits.
4. Under **Trusted server host keys**, add at least one key with **Add trusted host key**: its **Host-key algorithm** (`ssh-ed25519`, `ecdsa-sha2-nistp256`, `ecdsa-sha2-nistp384` or `ssh-rsa`) and **SHA256 host-key fingerprint**. **Fetch current host key** contacts the server without sending credentials and shows the key it presents as **Observed, untrusted host key**. To use it you must either tick **I verified this fingerprint through a trusted, separate channel**, or accept it unverified with **Trust this first observed key (TOFU); I accept the first-connection identity risk**. Once saved, a different key is refused.

   ![The Trusted server host keys section of a new SSH tunnel: its explanation, Fetch current host key and Add trusted host key.](/screenshots/admin-access/ssh-tunnels-pins.webp){.mw-crop}

5. Under **Allowed destinations**, add each host and port the tunnel may open (**Destination hostname**, **Destination port**). Tick **Allow this destination on a private or loopback address** for an internal destination. At most 128 destinations and 16 host keys.

   ![The Allowed destinations section of a new SSH tunnel with one destination row: Destination hostname, Destination port 443, the private-address checkbox, Remove destination and Add destination.](/screenshots/admin-access/ssh-tunnels-targets.webp){.mw-crop}

6. Under **Connection limits**, adjust **Keepalive interval (seconds)**, **Connection timeout (milliseconds)** and **Maximum channels** if needed.
7. Tick **Enable this tunnel** and choose **Save tunnel**. An enabled tunnel needs a credential, at least one host key and at least one destination.

After saving, **Test saved connection and first destination** signs in to the SSH server and opens a channel to the first allowed destination. It checks the SSH sign-in and that the destination accepts a connection, not the destination's own protocol; hosting-panel diagnostics test the panel itself. Diagnostics always use the saved configuration, so save first. A failed test shows the **Diagnostic phase** and **Diagnostic result code**. Inspections and tests are limited to 6 per minute per administrator.

## How destinations are checked

- Destination names are resolved by the Mailwoman server and checked before each channel opens. If any resolved address is not allowed (a private or loopback address without permission, a cloud metadata address or another reserved range), the channel is refused.
- A private destination is allowed only when both the tunnel's destination and the hosting connection allow private addresses.
- HTTPS to the hosting panel keeps the original host name for certificate checking; the tunnel only carries the bytes.

## Change or remove a tunnel

- Changing the SSH server, port, username or host keys while keeping the saved credential asks you to **Review credential authority change** and **Confirm authority and save**, so a saved credential is not sent to a server you did not intend.
- The saved credential is sealed with the server key and never shown again ("A credential is saved and sealed. Its secret is never returned."). Leaving **Saved credential** unchanged keeps it.
- Each save checks that nobody changed the tunnel since you opened it; if someone did, reload and try again.
- **Remove tunnel** is refused while a hosting connection still uses the tunnel. Connections that used a removed tunnel stop working; they do not fall back to a direct connection.
- Disabling or changing a tunnel stops new channels at once; data already sent through an open channel is not recalled.

Tunnels and their credentials are stored in this server's database and are not included in a migration between database types. Back up the database and the server key together.

## Related

- [Hosting panel connections](./hosting.md)
- [Outbound connections](../security/egress.md)
