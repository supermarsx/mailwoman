---
title: Attachments
screen: true
---
# Attachments

The **Attachments** screen lists the files attached to the messages in your account, so you can find a file without remembering which message it came with. Choose **Attachments** in the sidebar. It opens in its own tab.

![The Attachments screen: the filter form on top and a table of four files with their type, size, sender, subject and date.](/screenshots/user-mailbox/attachments-screen.webp){.mw-shot}

1. The filter form: a search box, **Search entire account**, **Clear filters** and **More filters**.
2. The table of files. The count above it, here **4 files**, says how many match.
{.mw-callouts}

## Find an attachment

1. Open **Attachments**.
2. Type in **Search attachments**. The box takes file names, senders and subjects, and also the operators `filename:`, `from:`, `subject:` and `larger:`, as in `filename:report from:alice larger:1mb`.
3. The table narrows to the files on the page. To search every message in the account, choose **Search entire account**. While it runs, a line counts the files found, and you can cancel it. A search shows at most the first 1,000 matching files; narrow your filters to find more.
4. To start again, choose **Clear filters**.

Browse with **Next** and **Previous** at the bottom of the table. Filters apply to the page you are on; use **Next** to reach older messages.

## Narrow the list

Choose **More filters** to open the other filters.

![The attachment filters with More filters open: sender, message subject, folder, received date, file size, file name and file extensions.](/screenshots/user-mailbox/attachments-filters.webp){.mw-crop}

- **Filter by type**: for example **Images**, **PDFs**, **Documents**, **Spreadsheets**, **Archives** or **Other files**.
- **Sender** and **Message subject**.
- **Folder**: **All folders** or one folder.
- **Received**: **Any time**, **Past 7 days**, **Past 30 days**, **This year** or **Custom date range** with **Received from** and **Received through** dates.
- **File size**: **Any size**, **Up to 1 MB**, **1–10 MB**, **10 MB and above** or **Custom size range** with **Minimum size (MB)** and **Maximum size (MB)**.
- **Search in**: **All attachments** or **Current page**.
- **File name** and **File extensions**, for example `pdf, docx, xlsx`.

Active filters show as chips with a **Remove filter** button each.

## The table

![The table of attachments with sortable column headings File, Type, Size, Sender, Subject and Received, and one row per file.](/screenshots/user-mailbox/attachments-table.webp){.mw-crop}

1. The column headings: **File**, **Type**, **Size**, **Sender**, **Subject** and **Received**. Click a heading to sort by it. **Received** is sorted with the newest first.
2. The file name. Click it to open the file.
3. The subject. Click it to open the message that contains the file.
{.mw-callouts}

## Open or download a file

Click a file name in the table, or choose **Open** from a file's menu in a message ([Reading messages](/user/mailbox/reading)). A viewer opens in its own tab.

![A PDF opened in its own tab: the file name, a Download button and an information button at the top right, and the PDF viewer with its toolbar.](/screenshots/user-mailbox/attachments-preview.webp){.mw-shot}

The file name is at the top left. At the top right, one button downloads the file and another shows information about it. A PDF has the viewer's own toolbar with page navigation, search, zoom, annotation tools, print and save.

Files that can open in the application are images, PDF documents, text files, HTML documents, code and data files, audio, video, email messages and calendar files. A file larger than the preview size limit, or of another type, keeps its download button. Your administrator's attachment restrictions and download permissions still apply. If downloads are turned off, the file says **Downloads are turned off**.

## Options

| Label | What it does | Default |
|---|---|---|
| **Enable file previews** | Chooses which files can open in the application and which tools appear ([`reader.filePreviews`](/admin/settings#reader.filePreviews)) | on |
| **Maximum preview size (MiB)** | Files larger than this keep their download button but do not open a preview; text is also limited to 512 KiB | 64 |
| **Download buttons on attachments** ([`attachments.bar.showDownloadButtons`](/admin/settings#attachments.bar.showDownloadButtons)) | When each file shows its download button: **Always**, **On hover or focus** or **Never** | **On hover or focus** |

These are in **Message display** in Settings ([Mail settings](/user/settings/mail)).

## Troubleshooting

- **"No attachments match."**: no file on the page matches the filters. Choose **Clear filters**, or **Search entire account**.
- **"Search interrupted"**: the search of the whole account did not finish. Choose **Retry**.
- **"This image could not be decoded within the preview limits. Download the original to open it in a compatible application."**: download the file instead.
- **"The attachment policy blocks every file here"**: your administrator's policy blocks these files.

## Related

- [Reading messages](/user/mailbox/reading)
- [Searching](/user/mailbox/search)
- [Writing messages](/user/mailbox/composing)
- [Mail settings](/user/settings/mail)
