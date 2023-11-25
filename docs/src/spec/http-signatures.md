# HTTP signatures

HTTP signatures are used by Kitsune to validate that the Activity/Object actually originates from the user it claims to.  
We implement a subset of [`draft-cavage-http-signatures-12`](https://datatracker.ietf.org/doc/html/draft-cavage-http-signatures-12) to do this.

Only asymmetric cryptographic algorithms are implemented since the symmetric ones:

1. Could lead to potential key-confusion attacks
2. Aren't useful in the context of ActivityPub

We make use of the `keyId` field by looking up the public key material via this ID. The ID is sourced from the ActivityPub actor.  
The signature scheme used is inferred by the OID embedded in the public key material. The material is expected to be an X.509 SPKI structure.

At the moment, Kitsune uses RSA keys but has support for implementations that use Ed25519 for signatures.

> We use RSA because Mastodon doesn't support anything else. So if you want compatibility with Mastodon, you have to use RSA.  
> As soon as a mainline Mastodon release gets support for Ed25519 signatures, we will release an update that allows to rekey all the users.

But if you are happy to just federate with Kitsune users, you can use Ed25519!
