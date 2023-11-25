# Captcha

Kitsune offers the ability to require captchas on sign-up to protect your service against spam waves.

We offer different implementations to fit your specific needs

## hCaptcha

The rather well-known [hCaptcha service](https://www.hcaptcha.com/) advertises itself as a more privacy-oriented alternative to Google's reCaptcha.

To use it to protect your instance, add the following to your configuration:

```toml
[captcha]
type = "hcaptcha"
verify-url = "[Verify URL]"
site-key = "[Your site key]"
secret-key = "[Your secret key]"
```

## mCaptcha

[mCaptcha](https://mcaptcha.org/) is a lesser known open-source self-hostable captcha service.  
Technically it isn't a "captcha" and more of a "proof-of-work verification system", but it should defend your service against large spam attacks.

To use mCaptcha, add the following to your configuration:

```toml
[captcha]
type = "mcaptcha"
widget-link = "[Widget link]"
site-key = "[Your site key]"
secret-key = "[Your secret key]"
verify-url = "[Verify URL]"
```
