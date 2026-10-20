# Questions and answers

## Using TypstDrive

??? question "Do I need to know Typst?"
    No. The toolbar has buttons for headings, bold, lists, math, tables and images, and the preview shows the result straight away. [A short Typst primer](start/typst-primer.md) covers the basics in a few minutes.

??? question "Do I need to save?"
    No. Everything is saved automatically every few seconds. Pressing ++ctrl+s++ saves a named point in the [version history](guide/collaboration.md#version-history) that you can go back to.

??? question "Can I use templates from Typst Universe?"
    Yes. Import them as usual, for example `#import "@preview/charged-ieee:0.1.4": ieee`. You can also publish your own templates as [packages](guide/packages.md) for your server.

??? question "My font is not working."
    Use the family name shown in the editor's font picker, not the file name. Upload every style you use, such as bold and italic. See [Fonts and images](guide/fonts-and-images.md).

??? question "I shared a document, but the other person sees missing images."
    The images are probably not shared with them yet. Open the paperclip button in the editor and choose **Share these files**. See [Document files](guide/sharing.md#document-files).

??? question "Can I work offline?"
    Yes, with the [Typst Desktop](guide/desktop.md) app for Linux. It keeps documents on your computer and syncs them when you are back online.

??? question "I forgot my password."
    Choose **Forgot password** on the sign-in page if your server has email set up. Otherwise ask an administrator to set a new one.

## Running a server

??? question "How many people can one server handle?"
    A small server with 1 CPU and 1 GB of memory is fine for a handful of people writing at the same time. Add more memory for larger groups. For very large installs, see [Bigger installs](self-hosting/scaling.md).

??? question "Everyone gets signed out when I restart."
    Set `COOKIE_SECRET` to a fixed value. See [Install with Docker](self-hosting/install.md#2-create-a-secret).

??? question "Live editing does not work behind my proxy."
    Your reverse proxy needs to pass WebSocket connections through. See [HTTPS and reverse proxies](self-hosting/reverse-proxy.md).

??? question "Sign in with Google or my company login fails."
    Check that `PUBLIC_URL` matches the address in the browser exactly, including `https://`, and that the redirect URL registered with the provider matches the one shown in **Site Settings → Authentication**.

??? question "Emails never arrive."
    Use **Send test** in **Site Settings → Email** to see what goes wrong. Check your spam folder, and for Gmail make sure you use an app password.

??? question "How do I back up?"
    Copy the data folder, or dump your database. See [Updates and backups](self-hosting/updates-and-backups.md).

## About the project

??? question "Is TypstDrive made by the Typst team?"
    No. TypstDrive is a community project and is not affiliated with, endorsed by, or supported by the official Typst project. Please report problems on the [TypstDrive issue tracker](https://github.com/SirBlobby/TypstDrive/issues). See [About and disclaimer](about.md).

??? question "Is it free?"
    Yes. TypstDrive is open source under the Apache License 2.0.
