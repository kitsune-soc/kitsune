# Emailing

Configuring an Email server for Kitsune automatically enables account confirmation via Email.

The mailer currently only supports SMTP, no provider-specific REST APIs.

## Example

```toml
[email]
from-address = "kitsunemailer@joinkitsune.org"
host = "your.smtp.host"
username = "admin"
password = "password"
starttls = false
```

There is also an option config you can place in front of "from_address" if your email service provider does not do TLS over 465 and instead uses 587 (STARTTLS).

Here is an example configuration utilizing STARTTLS:

```toml
[email]
from_address = "kitsunemailer@joinkitsune.org"
host = "your.smtp-host.example"
username = "admin"
password = "password"
starttls = true
```

## Parameters

```starttls``` :

By default Kitsune users the port 465 to send mail. 

Most service providers use this port, but some (for example Postmark) need to have their TLS usage bootstrapped via STARTTLS over port 587. 

```from-address``` :

This is the address Kitsune puts into the `From` field of the Email

```host``` : 

This is the domain that your SMTP server is reachable under.

```username, password``` :

The credentials to your SMTP server. Which values to put here vary from provider to provider.
