# [Setup Manual](https://ritze03.github.io/ModSync) 

# ModSync

**ModSync** is a pre-launch tool for Minecraft that automatically synchronizes mods before starting the game. It ensures that required mods are downloaded and unwanted mods are removed.

The goal of this project is to make it easy to push updates to your modpack. Your friends only need to set up ModSync once on their systems. After that, you can push updates by editing a single file.  
No self-hosting required.

## How it works
Everyone who plays on your server needs to set up a pre-launch command once using the [Interactive Setup Manual](https://ritze03.github.io/ModSync).

Modrinth, Prism, and other launchers will run the program before the game starts.

The program fetches the list of mods from the specified **"Mods URL"**. Mods are downloaded or removed according to the contents of that file.

ModSync runs ➡ ModSync fetches the mod list from the "Mods URL" ➡ ModSync checks the installed mods and updates them accordingly ➡ ModSync exits ➡ The game is launched.

## How to set up
#### Prerequisites
* A way to host the mod list that allows you to edit it and provides access to the raw text file (GitHub or any similar service).
* Get [**ModSyncUI**](https://github.com/Ritze03/ModSyncUI) to create and manage the mod list quickly and easily.

#### 
Upload your mod list and copy the raw text URL (GitHub example: https://raw.githubusercontent.com/****************). This will be your **"Mods URL"**.

Send your friends the setup guide and the Mods URL.  
Done.

## Setup (on the server)
If you host your Minecraft server on a root server, you can set up ModSync there as well.

Download ModSync the same way you would on a client.

In your launch script, add:  
`"/home/<username>/ModSync" --modsurl <Mods URL> --path <Path to your server instance> --cli`

**--path** is redundant if your working directory is already inside the server instance.

# TODO
* Add a transaction summary screen with a 5-second timeout.
* Add an "Env" option to define whether a mod is meant for the server or the client.
* Add human-readable names to the file format.
* Add support for resource packs and other patches (similar to Modrinth: `mods/filename`, `resourcepacks/filename`).

---

## Features

- Automatically download required mods before launch.  
- Remove mods marked for deletion.  
- Verify file integrity using optional SHA256 hashes.  
- Supports both **GUI** and **CLI** modes.  
- Generate SHA256 hashes of local files.  

---

## Installation

Build from source with Rust:

```bash
git clone https://github.com/Ritze03/ModSync.git
cd ModSync
cargo build --release
```

The compiled binary will be in `target/release/ModSync`.  

---

## Usage

### CLI Options

```text
--modsurl <URL>       URL of the remote mod list (cannot be used with --modsfile)
--modsfile <PATH>     Local file containing the mod list (cannot be used with --modsurl)
--path <PATH>         Path to the modpack root (default: current directory)
--cli                 Run in CLI mode instead of GUI
--hash <FILE>         Generate SHA256 hash of a file and exit
```

---

### Mod List Format

ModSync expects a mod list in the following format (one mod per line):

```
# Category | ModName | DownloadURL | SHA256
```

- **Category:** `REQUIRED` or `REMOVE`  
  - `REQUIRED`: Automatically downloaded; required for the game to run.  
  - `REMOVE`: Deletes the specified mod from the local mods folder.  

- **ModName:** The filename of the mod JAR.  

- **DownloadURL:** URL to download the mod (ignored for `REMOVE` entries).  

- **SHA256:** Optional SHA256 hash for file verification (ignored for `REMOVE` entries).  

**Example:**

```
REQUIRED | example-mod.jar | https://example.com/mods/example-mod.jar | abc123...
REMOVE   | old-mod.jar     |                       |
```

---

### Examples

#### Run with a remote mod list in GUI mode:

```bash
modsync --modsurl https://example.com/modlist.txt
```

#### Run with a local mod list in CLI mode:

```bash
modsync --modsfile mods.txt --cli
```

#### Generate a SHA256 hash for a file:

```bash
modsync --hash path/to/mod.jar
```

---

## License

MIT License. See [LICENSE](LICENSE) for details.  
