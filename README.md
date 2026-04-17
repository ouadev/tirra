<div align="center">

<img src="resources/icon-2.png" width="140px" />

# Tirra

[![License](https://img.shields.io/crates/l/iced.svg)](https://github.com/iced-rs/iced/blob/master/LICENSE)

Encrypted Notebook written in Rust.

  <div><img src="resources/screenshot-01.png" width="100%"></div>
  <div><img src="resources/screenshot-02.png" width="100%"></div>

</div>


## Features

* Browse, read, and edit text entries stored in an encrypted SQLite database
* Open file format
* on-the-fly encryption powered by sqlite3 encryption extension [tirravfs](https://github.com/ouadev/tirravfs)
* encryption scheme: ChaCha20-Poly1305. Key derivation algorithm: pbkdf2/Sha256, with 64007 iterations.
* Application is automatically closed after 3 minutes of inactivity
* Full-text search across all entries
* Option to manually change the SQL query used to read entries
* Entry operations: add, remove, edit content, and change creation_date.
* Sort by creation or modification date.
* Command-line interface to interface with the encrypted database.
* periodic saving of the content of the current entry.


## Keyboard Shortcuts

* CTRL + N : start new note.
* CTRL + S : save the current note.
* CTRL + L : switch between light and dark mode.
* CTRL + P : Show/hide SQL request and delete button.
* CTRL + SHIFT + F: Find.

## File Format

SQLite schema V0:

```
TABLE entries (
    id          INTEGER PRIMARY KEY,
    date_create INTEGER NOT NULL,
    date_modify INTEGER NOT NULL,
    type        INTEGER,
    text        BLOB
            )

TABLE information (
    id              INTEGER PRIMARY KEY,
    schema_ver      INTEGER,
    local_commit    BLOB,
    origin_commit   BLOB,
    local_source    BLOB,
    origin_source   BLOB,
    local_ts        INTEGER NOT NULL,
    origin_ts       INTEGER NOT NULL
)
```



+ *type* is the type of the entry, for now it is set to 0 (General Type).
+ *text* is the content of the note. The title is not stored separately and is calculated by the reader from the first sentences.
+ *date_create* and *date_modify* are dates of creation and last modification. They are UNIX epoch timestamps. UTC.
+ *information* table is meant to be used as a basis for synchronization. Any change to the database is recorded in this table. Synchronization is not yet supported


## Platforms

Primarily developed and tested on Linux. Windows and macOS support is experimental.


## Command Line 

``` 
launch GUI:
$ tirra [DB_FILE]

if no databse file is provided, the default one will be attempted: [HOME]/awal.tirra


launch Command Line Interface:
$ tirra-cli COMMAND [ARGUMENTS]
OR also (in unix systems):  
$ tirra --cli COMMAND [ARGUMENTS]

Usage:

tirra-cli ver
tirra-cli add		DB_FILE DATE < CONTENT_FILE
tirra-cli delete	DB_FILE ID
tirra-cli stat  	DB_FILE
tirra-cli decrypt	DB_FILE OUT_FILE
tirra-cli encrypt	PL_FILE OUT_FILE 


COMMANDS
       ver    Display the version information and exit.
       add    Add content from a file using redirection. DATE must
              be a Unix epoch timestamp.
       delete Remove an entry from the database by its ID.
       stat   Display statistics about the database.
       decrypt
              Decrypt an encrypted database file.
              it can be used to decrypt any file encrypted using Age/scrypt.
       encrypt
              Encrypt a plaintext database file.
              it can encrypt any file using Age/scrypt.

ARGUMENTS
       DB_FILE
              Path to an encrypted Tirra database file.
       DATE   Unix epoch timestamp.
       PL_FILE
              Path to a plaintext file.
       OUT_FILE
              Output file, either of encryption or decryption.
       ID     id of the entry.
       CONTENT_FILE
              A file whose content will be added to the database via
              standard input redirection.
```



## Important Notes

* Backup: This application does not include automatic backup functionality. Manual backups are recommended.
* Version v2.x.x uses SQLite extension *tirravfs* for encryption. Instead of the v1.x.x approach of decrypting the database file to disk — leaving it briefly accessible in plaintext — v2.x.x keeps all plaintext data exclusively in memory. Databases created with v1.x.x are not compatible with v2.x.x. Migration is possible.


<div align="center"><a href="https://github.com/iced-rs/iced">
  <img src="https://gist.githubusercontent.com/hecrj/ad7ecd38f6e47ff3688a38c79fd108f0/raw/74384875ecbad02ae2a926425e9bcafd0695bade/color.svg" width="130px">
</a>
</div>