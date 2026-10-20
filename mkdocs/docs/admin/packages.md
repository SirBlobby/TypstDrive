# Packages

**Site Settings → Packages** controls the server's [package library](../guide/packages.md).

| Setting | Default | What it does |
|---|---|---|
| Allow package publishing | On | When off, nobody but administrators can publish. Existing packages keep working. |
| Only administrators can publish | Off | Everyone can still use packages, but only administrators publish them. |
| Owners can delete their packages | On | When off, only administrators can delete packages. |
| Allow private packages | On | When off, new packages must be public. Packages that are already private stay private. |
| Default for new packages | Public | Used when the person publishing does not choose. |
| Maximum package size | No limit | Total size of one published version. `0` means no limit. |
| Maximum versions per package | No limit | Publishing stops at this many versions. `0` means no limit. |

Administrators can always publish and delete packages and see private ones, but size and version limits apply to them too.
