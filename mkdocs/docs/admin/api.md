# API access

The [API](../api/index.md) lets people's own programs and scripts turn Typst into PDFs and images, and read their documents. **Site Settings → API Access** decides who can use it and how much.

## Default limits

These apply to every account without its own limits.

| Setting | Default | What it does |
|---|---|---|
| API access | On | Whether people can create API keys at all. |
| Rate limit | 60 | Requests per minute for each key. People can set a lower limit on their own keys. |
| Maximum keys | 10 | Keys each person can have. |
| Daily quota | 0 | Requests per person per day (UTC). `0` means no limit. |

## Limits for one person

The user list shows each account's access, limits, keys and requests today. **Edit limits** gives one person different limits. Leave a field empty to keep the default, or use **Reset to defaults**. Accounts with their own limits are marked **Custom**.
