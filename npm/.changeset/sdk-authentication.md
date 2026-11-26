---
"@localix-ai/flowapi": minor
---

SDK authentication: the generated client sends every kind of security scheme as OpenAPI and RFC 6749, 6750 and 7617 say (API keys, HTTP bearer and basic, other HTTP schemes, OAuth 2.0 and OpenID Connect tokens, and OAuth 2.0's client credentials grant run by the client itself), follows `security` exactly (alternatives in order, `{}`, `[]`, an operation's own over the description's, Appendix F), sends credentials only over TLS or to a loopback host unless `allowInsecureCredentials` is set, and throws `CredentialError`, `InsecureCredentialError` or `TokenError` before sending what it cannot. `flowapi-sdk.yml` declares the schemes and `security` a description lacks (`auth.<name>.type`, `security`, a method's `security`); `config check` has the `config.auth.*` and `config.security.unknown` codes, `config draft` names a variable for every scheme, and the generated repository's mock, tests and README cover each.
