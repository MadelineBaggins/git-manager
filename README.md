# Maddi's Git Manager

Hi there! This is a piece of software I wrote for myself
to manage my own git repositories on a private server.

I do a lot of work on my own hobby projects by myself, so
I didn't need anything collaborative or too complicated.
Plus with GitHub being a bit shady with training AI I liked
the idea of having my own server.

## Function

With `git-manager`, you have a central `admin` repository
that contains a `config.xml` file. This file defines the
layout of all your git repositories. Here you can create
new repositories by cloning the repository, adding a new
repository to `config.xml`, and pushing the changes back
to the remote. A `post-receive` hook then runs
`git-manager switch` which causes `git-manager` to update
your server's state to match that described in `config.xml`.

`git-manager` places all your repositories in a central
`store` directory, then symlinks those directories out to
other directories. This allows you to keep your repositories
in a central directory, making things like backups easier,
while making it easy to create a sensible file structure
that makes it simpler to find the repository you're looking
for. 

`git-manager` makes it easy to manage remote hooks too,
allowing you to define `pre-receive`, `update`, and
`post-receive` hooks. This is useful for doing things like
forwarding on changes to GitHub for example.

`git-manager` also has a `git-manager search` command that
allows you to search for repositories and is planned to be
compatible with my upcoming `smartget` project.


## Installation

To install the `git-manager` binary on your git server, make
sure cargo is installed on your git server (I know,
overkill), then run `cargo install maddi-git-manager`. This
will download and install the binary. I'll eventually
package this in some common distributions, but until then,
this is the best way to go about it.

Then run `git-manager init server`. It will prompt you to
add flags that define the default branch name, in addition
to the store directory and a root directory for symlinks to
be created.

## Use

**under construction**

## License

This software is licensed under the GPLv3. The full text
of this license can be found in `LICENSE` at the project's
root.

I'm more than happy to relicense my code for other open
source projects so just reach out if you need something
more permissive.

*Copyright 2025 Madeline Baggins <declanbaggins@gmail.com>*
