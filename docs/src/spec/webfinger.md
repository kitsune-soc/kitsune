# Webfinger

Kitsune uses Webfinger for resolving mentions and such. For example, when mentioning a user in a post in the form `@user@instance.org`, the mention is interpreted as a Webfinger `acct` query.  
We then connect to the remote server and send an HTTP GET request on the path `/.well-known/webfinger` with the query parameter `resource` set to `acct:user@instance.org`.

The server is then expected to return a Webfinger resource containing a link with the `rel` property set to `self` and the `href` attribute pointing to the ActivityPub actor.

Example Webfinger resource:

```json
{
    "subject": "acct:user@instance.org",
    "aliases": [],
    "links": [
        {
            "rel": "self",
            "href": "https://instance.org/users/user"
        }
    ]
}
```
