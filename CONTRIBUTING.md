# List of rules and norms

## Bug Reports & Feature Requests
- Bug reports and feature requests are greatly appreciated and can be done in form of an [issue](https://github.com/JakeRoggenbuck/auto-clock-speed/issues)
- The most helpful reports generally follow the bug report template. One tip, is to run acs with the `-c` to show exactly what the latest commit the executable is running with.
- Bugs always get moved to the soonest release

## Develop
If you would like to help complete a task, comment in the issue and we may assign it to you!

## Pull Requests
- Pull requests must be tested but with `cargo test` and ran on local machine (laptop with battery)
- Pull requests that change core functionality must have a review
- Please make WIP PRs drafts

![mark as draft](https://user-images.githubusercontent.com/35516367/152289665-76631734-fbe4-41e6-9b6e-6a7019fa6ff4.png)

## Testing
- Tests must be done by contributors of the project
- Tests must both test how acs preforms with root and normal permissions

- How to run tests
	```sh
	cargo build && ./target/debug/acs monit -c 
	cargo build && ./target/debug/acs run -c 

	cargo build && sudo ./target/debug/acs monit -c 
	cargo build && sudo ./target/debug/acs run -c 
	```

	```sh
	cargo test

	cargo test unit	# unit testing, none platform specific
	cargo test acs	# laptop specific testing
	```

## Issues marked with `help wanted` or `good first issue`
- Issues marked with either `help wanted` or `good first issue` are a great place to start!
- Issues marked with these are a great place for discussion and user suggestions

## Projects tab
- The [projects tab](https://github.com/JakeRoggenbuck/auto-clock-speed/projects/1) is something we use extensively to organize when things get done

## Milestones & Version
- Milestones are how we prepare for a release and version number increment
- We will add 10-20 issues to a milestone to be worked on
- Issue move around milestones as we change our requirements and priorities
- Bugs always get added to the closet milestone

## Releases
- Before a release, we test on several computers making sure they all function, and all function the same
- We add a git tag to the branch and push it
- Then we draft and publish the release
- We typically release a new numbered version every one to two months, usually with 10-20 new features or fixes

## Future
- We plan to add a wiki page
- We plan to do pre-releases
