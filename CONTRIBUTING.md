# Contributing to RampMaker

## Introduction

This file documents the procedures for developing the RampMaker project. It targets contributors and maintainers. Contributions, be it in the form of issues or pull requests, are always welcome, so don't be shy!

At this point, this document is far from a comprehensive guide. It will be extended over time. Please open an issue, if anything is unclear, or if you need information not present here.


## Publishing a release

This procedure should really be automated, but for now it is at least documented.

1. Make sure your local `main` branch is up-to-date:

``` bash
git switch main
git pull --rebase
```

2. Choose new version number, according to [Semantic Versioning]. The following steps will use `a.b.c` as a placeholder for the chosen version.

3. Create a release branch (replace `a.b.c` with actual version)

``` bash
git switch -c publish-a.b.c
```

4. Update changelog: Go through all pull requests since the last release and mention the relevant ones. Use existing changelog entries as the template. Commit this to the repository.

5. Update version in top-level `Cargo.toml` and README.md. Commit changes.

6. Push branch, open a pull request. This makes sure the CI runs and gives other maintainers a chance to weigh in.

7. Once ready, publish crate: Run `cargo publish` from the repository root.

8. Tag the release

``` bash
git tag va.b.c
```

9. Merge pull request, clean up your local branch.

``` bash
# missing: merge pull request
git switch main
git pull --rebase
git remote prune origin
git branch -d publish-a.b.c
```

10. Push the release tag

``` bash
git push --tag
```


[Semantic Versioning]: https://semver.org/
