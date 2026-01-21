# [Setup Manual](https://ritze03.github.io/ModSync) 

# ModSync

**ModSync** is a pre-launch tool for Minecraft that automatically synchronizes mods before starting the game. It ensures that required mods are downloaded, and unwanted mods are removed, helping you keep your mod folder consistent and up to date.  

## TODO
* Add transaction summary screen with a 5 second timeout to view.
* Add "Env" to define if the mod is meant for the server or the client.
* Add human readable names to the file format
* Add Support for Resource Packs and other patches (do it like modrinth: mods/filename, resourcepacks/filename).
---

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
