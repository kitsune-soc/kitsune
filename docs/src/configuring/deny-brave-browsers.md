# Deny Brave Browsers

Brave is a company founded by a queerphobic bigot, funded by people such as Peter Thiel.
You can get pretty much all of Brave's "privacy advantages" with a Firefox + uBlock Origin + DuckDuckGo installation. 

(Plus, Brave is another Chromium-based browsers. Using Firefox helps diversify the browser landscape at least a little bit)

When this setting is enabled, all browsers with the Brave User-Agent are redirected to an [article explaining the hateful and problematic background of Brave](https://www.spacebar.news/stop-using-brave-browser/).  

Since Kitsune is about choice, we give you the ability to simply toggle this functionality off.  
While we give you that option, you maybe want to keep this option on.

The reasoning behind this is simple:

- If people aren't aware and care, they can switch browsers. Switching a browser isn't a herculean task.  
- If people are aware and don't care, I'm not sure if you want them on your service.

```toml
[server]
deny-brave-browsers = true
```
