# Storage and history

**Site Settings → Content → Storage** shows how much space the whole server uses, and how much each account and team uses, largest first.

## Storage limits

- **Default quotas** set the most each person, and each team space, can store. Use `0` for no limit.
- **Usage by account** lets you search for someone and give them their own limit. Leave it empty to use the default.

When someone is full, new uploads are blocked. Nothing they already have is removed.

## Version history

**Version history** sets how many versions each document keeps. When a document has more, the oldest ones are deleted. Lowering the number tidies every document straight away. Use `0` to keep every version.

Automatic versions saved within 15 minutes of each other are already merged into one, so history does not fill up with tiny saves.

## Trash

**Trash** sets how many days deleted items are kept before they are removed for good. The default is 30 days, and the most is 365.

- Deleted documents, projects, folders and files wait in their owner's [trash](../guide/drive.md#trash), where they can be restored. They still count toward storage.
- The server checks every hour and removes anything older than the limit. Lowering the limit applies to items already in the trash.
- Use `0` to turn the trash off. Deleting then removes items straight away, and anything already in the trash is removed within the hour.

Changing the setting is recorded in the audit log.
