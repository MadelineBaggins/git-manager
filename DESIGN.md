# Git-Manager Design

- Git-Manager will support multiple remotes
- Each remote will contain a cache for fuzzy searching
  - Perhaps a small sqlite3 file we can fetch
- Each remote will have its own config file stored in the
  admin repository. The admin file lists all the remote
  files.
- Each client will have a config file that contains a list
  of named locations of admin files. If only one location
  is specified, the remote can be left out in the commands.
- The CLI will communicate to the server through pulling and
  pushing the local admin repository.
- The CLI will be basic and the rest of the functionality
  will be through a TUI.
- The TUI will support fuzzy searching for files and
  repositories using metadata and allow for cloning
  repositories or pulling individual files down over `scp`.
- The TUI will include a *queue* so multiple files or
  repositories may be pulled down.
- The TUI will also allow for modifying the tags on a
  repository.
- The server binary will be the same as the client binary.
- Repository hooks can be managed by the admin repository
- The cli will allow for backing up and restoring entire
  remotes in tarballs too for moving from server to server.
- The cli should have commands for gathering lists of
  repositories that match certain criteria for use in
  scripts.
  - `<cli> list --tag="lake-blog` to list the
  repositories with that tag.

## To-do

- [ ] Refactor the current working directory
- [ ] Implement the multi-file XML config library
  - Write using a recursive descent parser

## The Admin Repository

### config.xml

```xml
<config store="/home/git/repositories">
  <!-- The repository used to manage git-manager. -->
  <repo id="admin">
    <tag>META</tag>
    <alias>admin</alias>
    <hook name="post_update">
      cd ..
      git --git-dir=.git reset --hard
      /usr/bin/git-manager switch --config=config.xml
    </hook>
  </repo>
  <!-- My personal website. -->
  <repo
    id="2025-03-09-personal-website"
    src="website/repo.xml"
  />
  <!-- Some random scratch repositories. -->
  <div src="repos/scratch.xml"/>
</config>
```

### website/repo.xml

```
<tag>web</tag>
<tag>website</tag>
<tag>resume</tag>
<tag>portfolio</tag>
```
