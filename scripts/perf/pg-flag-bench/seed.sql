INSERT INTO accounts(id,kind,host,port,tls,username,sealed_creds)
 SELECT 'acct'||a,'imap','h',993,'tls','u'||a,'\x00'::bytea FROM generate_series(1,64) a;
INSERT INTO mailboxes(id,account_id,name,role,uidvalidity,total,unread)
 SELECT 'mbox'||a,'acct'||a,'INBOX','inbox',1,200,200 FROM generate_series(1,64) a;
INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,message_id,thread_id,internaldate,size,flags_json,envelope_json)
 SELECT 'm'||a||'-'||n,'acct'||a,'mbox'||a,n,1,'<'||a||'.'||n||'@x>','t'||a||'-'||n,
        to_char(timestamp '2026-01-01' + n*interval '1 minute','YYYY-MM-DD"T"HH24:MI:SS"Z"'),4000,'[]',
        convert_to(repeat('envelope-subject-and-addresses ',20),'UTF8')
 FROM generate_series(1,64) a, generate_series(1,200) n;
