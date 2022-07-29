# Disk scrub

## Abstract

This tool helps to check integrity of files. Fig. 1 illustrates key components that may cause file corruption.

![Abstract](charts/abstract.drawio.svg)

Fig. 1 What damages your important files?

### (1) User

Users might accidentally change or delete files.

### (2) Application

Applications might have defects that cause file corruption.

### (3) File system

Utilities of file system (such as chkdsk, fsck) may fix the file system errors and that may cause some files to be removed.

### (4) Hard drive

Hard drive's failure may make sectors unreadable. You may not be aware of this for a long period of time because errors cannot be detected until actually reading the sectors where the error is occurring.This means that files that have not been accessed for a long time may be corrupted without being noticed.

## How it works

This tool is focused on detecting file corruption. Fig.2 illustrates how the tool works.

1. It walks through all the files under the specified directory and calculate hash of each file.

1. It reads 'Controlfile' and compare the list of file and hash captured at the previous step.

1. It displays added, removed, and modified files.

1. Update 'Controlfile' with the lates result.

![Solution](charts/solution.drawio.svg)

Fig. 2 How it works

** Note: It is strongly recommended that you locate the 'Controlfile' in a fairly reliable location, e.g., on an SSD, or at least not on the same HDD being inspected.

## How to run

Grub the [package](https://github.com/ruimo/disk-scrub/releases) and unpack it and just run:

    $ ./disk_scrub /target/directory/to/inspect
    Summary:
      Added files: 4
      Removed files: 0
      Modified files: 0
    
    Details:
    [Added files]
      "disk_scrub"
      "disk_scrub-macos-x86_64.zip"
      "test/a"
      "test/b/c"
    [Removed files]
    [Modified files]

The 'Controlfile' will be created at the current working directory. You can change it by -f option.

$ ./disk_scrub -f /path/to/Controlfile /target/directory/to/inspect

The results are printed to standard output. Current version of this tool is:

- Has no functions for notification. You can use your favorite tools to send the report to mail/Slack/etc.

- Has no functions to execute periodically. You can use cron or any other tool for this. When launching the tool, please make sure the previous instance is not still running.

- It may be a good idea to use a tool such as ionice in conjunction with this tool so that to prevent disk scrub occupies disk access.

- Has no function to backup files.

- Has no function to retain older versions of a file. You may want to use some sort of backup solutions such as [Backblaze](https://www.backblaze.com/cloud-backup.html#af9uz7). You can recover files that you accidentally modified/deleted with these kind of solutions.