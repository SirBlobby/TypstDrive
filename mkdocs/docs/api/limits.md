# Limits and Caching

## Limits

Administrators set the limits for each instance and can change them per account. The defaults are:

| Limit | Default |
|---|---|
| Requests per minute, per key | 60 |
| Keys per account | 10 |
| Requests per account per UTC day | Unlimited |

Your current limits, key usage and today's request count are shown in **Settings → API Keys**.

- The per-minute limit is a fixed 60-second window for each key. When it is reached, requests return `429` until the window resets.
- The daily quota counts requests across all of your keys. When it is reached, requests return `429` until the next UTC day.
- If an administrator turns off API access for your account, requests return `403` and new keys cannot be created.

## Caching

Requests with identical code, format and inline files from the same account are served from a cache for up to one hour, without recompiling. Cached responses still count toward your rate limit and daily quota.

!!! note
    The cache key does not include your uploaded account files. If you replace an uploaded file, a cached render of the same code can be returned for up to an hour. Change the code or send the file inline to get a fresh render.
