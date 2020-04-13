# Contributing to Enseada
_Adapted from the [Atom Contribution Guide](https://github.com/atom/atom/blob/master/CONTRIBUTING.md)_

First off, thanks for taking the time to contribute!

The following is a set of guidelines for contributing to Enseada and its packages, which are hosted in the [Enseada Organization](https://github.com/enseadaio) on GitHub. These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

<!-- toc -->

- [Code of Conduct](#code-of-conduct)
- [I don't want to read this whole thing I just have a question!!!](#i-dont-want-to-read-this-whole-thing-i-just-have-a-question)
- [What should I know before I get started?](#what-should-i-know-before-i-get-started)
  * [Read the Guidelines](#read-the-guidelines)
- [How Can I Contribute?](#how-can-i-contribute)
  * [Reporting Bugs](#reporting-bugs)
    + [Before Submitting A Bug Report](#before-submitting-a-bug-report)
    + [How Do I Submit A (Good) Bug Report?](#how-do-i-submit-a-good-bug-report)
  * [Suggesting Enhancements](#suggesting-enhancements)
    + [Before Submitting An Enhancement Suggestion](#before-submitting-an-enhancement-suggestion)
    + [How Do I Submit A (Good) Enhancement Suggestion?](#how-do-i-submit-a-good-enhancement-suggestion)
  * [Your First Code Contribution](#your-first-code-contribution)
    + [Local development](#local-development)
  * [Pull Requests](#pull-requests)
  * [Sign your work](#sign-your-work)
- [Styleguides](#styleguides)
  * [Git Commit Messages](#git-commit-messages)
  * [Rust Code Styleguide](#rust-code-styleguide)

<!-- tocstop -->

## Code of Conduct

This project and everyone participating in it is governed by the [Enseada Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. <!-- Please report unacceptable behavior to [we-need-an@email.com](mailto:we-need-an@email.com). -->

## I don't want to read this whole thing I just have a question!!!

No worries! Just [open an issue](https://github.com/enseadaio/enseada/issues/new?assignees=&labels=question&template=question.md&title=) with the correct template and ask away.

## What should I know before I get started?

### Read the Guidelines

A good way to start grasping the codebase is reading our [Project Guidelines](GUIDELINES.md), which detail
a lot of conventions adopted by the project.

## How Can I Contribute?

### Reporting Bugs

This section guides you through submitting a bug report for Enseada. Following these guidelines helps maintainers, and the community understand your report :pencil:, reproduce the behavior :computer: :computer:, and find related reports :mag_right:.

Before creating bug reports, please check [this list](#before-submitting-a-bug-report) as you might find out that you don't need to create one. When you are creating a bug report, please [include as many details as possible](#how-do-i-submit-a-good-bug-report). Fill out [the required template](https://github.com/atom/.github/blob/master/.github/ISSUE_TEMPLATE/bug_report.md), the information it asks for helps us resolve issues faster.

> **Note:** If you find a **Closed** issue that seems like it is the same thing that you're experiencing, open a new issue and include a link to the original issue in the body of your new one.

#### Before Submitting A Bug Report

* **Check the [issue tracker](https://github.com/enseadaio/enseada/issues)** for a list of common questions and problems.
* **Gather the required information** as described by the [bug report template](https://github.com/enseadaio/enseada/issues/new?assignees=&labels=bug&template=bug_report.md&title=)

#### How Do I Submit A (Good) Bug Report?

Bugs are tracked as [GitHub issues](https://guides.github.com/features/issues/). Create an issue on the repository and provide the following information by filling in [the template](https://github.com/enseadaio/enseada/issues/new?assignees=&labels=bug&template=bug_report.md&title=).

Explain the problem and include additional details to help maintainers reproduce the problem:

* **Use a clear and descriptive title** for the issue to identify the problem.
* **Describe the exact steps which reproduce the problem** in as many details as possible. For example, start by explaining how you started Enseada, e.g. which command exactly you used in the terminal, with what configuration variables and in which environment.
* **Provide specific examples to demonstrate the steps**. Include links to files or GitHub projects, or copy/pasteable snippets, which you use in those examples. If you're providing snippets in the issue, use [Markdown code blocks](https://help.github.com/articles/markdown-basics/#multiple-lines).
* **Describe the behavior you observed after following the steps** and point out what exactly is the problem with that behavior.
* **Explain which behavior you expected to see instead and why.**
* **Include log entries** as they often contain useful information about what Enseada is doing at any time.
* **If the problem wasn't triggered by a specific action**, describe what you were doing before the problem happened and share more information using the guidelines below.

Provide more context by answering these questions:

* **Did the problem start happening recently** (e.g. after updating to a new version of Enseada) or was this always a problem?
* If the problem started happening recently, **can you reproduce the problem in an older version of Enseada?** What's the most recent version in which the problem doesn't happen? You can download older versions of Enseada from [the releases page](https://github.com/enseadaio/enseada/releases).
* **Can you reliably reproduce the issue?** If not, provide details about how often the problem happens and under which conditions it normally happens.
* If the problem is related to the storage engine (e.g. pushing or pulling packages), **does the problem happen for all storage providers or only some?** Does the problem happen only with one type of package or all of them?

Include details about your configuration and environment:

* **Which version of Enseada are you using?** You can get the exact version by running `enseada-server version` in your terminal, or by checking the [Docker image tag](https://hub.docker.com/r/enseada/enseada/tags) you are using.
* **What's the name and version of the OS you're using**?
* **Are you running Enseada in a virtual machine?** If so, which VM software are you using and which operating systems and versions are used for the host and the guest?
* **Are you running Enseada in a container orchestrator?** If so, which one? Is is installed on a public cloud, private cloud or on premise?
* **What flavour of CouchDB are you using?** E.g. Apache CouchDB, IBM Cloudant.

### Suggesting Enhancements

This section guides you through submitting an enhancement suggestion for Enseada, including completely new features and minor improvements to existing functionality. Following these guidelines helps maintainers and the community understand your suggestion :pencil: and find related suggestions :mag_right:.

Before creating enhancement suggestions, please check [this list](#before-submitting-an-enhancement-suggestion) as you might find out that you don't need to create one. When you are creating an enhancement suggestion, please [include as many details as possible](#how-do-i-submit-a-good-enhancement-suggestion). Fill in [the template](https://github.com/enseadaio/enseada/issues/new?assignees=&labels=enhancement&template=feature_request.md&title=), including the steps that you imagine you would take if the feature you're requesting existed.

#### Before Submitting An Enhancement Suggestion

* **Check if there's already [a similar suggestion](https://github.com/enseadaio/enseada/labels/enhancement) which provides that enhancement.**
* **Gather the required information** as described by the [feature request template](https://github.com/enseadaio/enseada/issues/new?assignees=&labels=enhancement&template=feature_request.md&title=)

#### How Do I Submit A (Good) Enhancement Suggestion?

Enhancement suggestions are tracked as [GitHub issues](https://guides.github.com/features/issues/). Create an issue on the repository and provide the following information:

* **Use a clear and descriptive title** for the issue to identify the suggestion.
* **Provide a step-by-step description of the suggested enhancement** in as many details as possible.
* **Provide specific examples to demonstrate the steps**. Include copy/pasteable snippets which you use in those examples, as [Markdown code blocks](https://help.github.com/articles/markdown-basics/#multiple-lines).
* **Describe the current behavior** and **explain which behavior you expected to see instead** and why.
* **Include API ProtoBuf** which help you demonstrate the new feature or point out the part of Enseada which the suggestion is related to.
* **Explain why this enhancement would be useful** to most Enseada users and isn't something that could be out of scope for the project.
* **List some other package registries where this enhancement exists.**
* **Which version of Enseada are you using?** You can get the exact version by running `enseada-server version` in your terminal, or by checking the [Docker image tag](https://hub.docker.com/r/enseada/enseada/tags) you are using.
* **Specify the name and version of the OS you're using.**

### Your First Code Contribution

Unsure where to begin contributing to Enseada? You can start by looking through these `good-first-issue` and `help-wanted` issues:

* [Good first issues][good-first-issues] - issues which should only require a few lines of code, and a test or two.
* [Help wanted issues][help-wanted] - issues which should be a bit more involved than `good-first-issue` issues.

Both issue lists are sorted by total number of comments. While not perfect, number of comments is a reasonable proxy for impact a given change will have.

If you want to read about using Enseada or developing integrations with Enseada, check out the official [docs](https://docs.enseada.io).

#### Local development

Enseada can be developed locally. For instructions on how to do this, check out the project [README](README.md) and the [Developer Guide](https://docs.enseada.io/developers/).

### Pull Requests

The process described here has several goals:

- Maintain Enseada's quality
- Fix problems that are important to users
- Engage the community in working toward the best possible Enseada
- Enable a sustainable system for Enseada's maintainers to review contributions

Please follow these steps to have your contribution considered by the maintainers:

1. Follow all instructions in [the template](.github/pull_request_template.md)
2. Follow the [styleguides](#styleguides)
3. After you submit your pull request, verify that all [status checks](https://help.github.com/articles/about-status-checks/) are passing <details><summary>What if the status checks are failing?</summary>If a status check is failing, and you believe that the failure is unrelated to your change, please leave a comment on the pull request explaining why you believe the failure is unrelated. A maintainer will re-run the status check for you. If we conclude that the failure was a false positive, then we will open an issue to track that problem with our status check suite.</details>

While the prerequisites above must be satisfied prior to having your pull request reviewed, the reviewer(s) may ask you to complete additional design work, tests, or other changes before your pull request can be ultimately accepted.

### Sign your work

The sign-off is a simple line at the end of the explanation for the patch. Your signature certifies that you wrote the patch or otherwise have the right to pass it on as an open-source patch. The rules are pretty simple: if you can certify the below (from developercertificate.org):

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.
1 Letterman Drive
Suite D4700
San Francisco, CA, 94129

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.


Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

Then you just add a line to every git commit message:

`Signed-off-by: Joe Smith <joe.smith@email.com>`

Use your real name (sorry, no pseudonyms or anonymous contributions.)

If you set your user.name and user.email git configs, you can sign your commit automatically with git commit -s.

## Styleguides

### Git Commit Messages

* Write your commit message in the imperative: "Fix bug" and not "Fixed
  bug" or "Fixes bug." This convention matches up with commit messages
  generated by commands like git merge and git revert.
* Limit the first line to 72 characters or less
* Reference issues and pull requests liberally after the first line
* When only changing documentation, include `[ci skip]` in the commit title
* Consider using the following message template:

```
[one line-summary of changes]

Because:
- [relevant context]
- [why you decided to change things]
- [reason you're doing it now]

This commit:
- [does X]
- [does Y]
- [does Z]

```

### Rust Code Styleguide

Enseada uses the linting rules provided by [Clippy](https://github.com/rust-lang/rust-clippy), the official Rust linter.

[beginner]:https://github.com/enseadaio/enseada/labels/good%20first%20issue
[help-wanted]:https://github.com/enseadaio/enseada/labels/help%20wanted