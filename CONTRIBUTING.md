# Contributing to Rust-FlightPlan

First off, thanks for taking the time to contribute.

The following is a set of guidelines for contributing to Rust-FlightPlan, which is hosted with GitLab.
These are mostly guidelines, not rules.
Use your best judgment and feel free to propose changes to this document in a merge request.

## Table Of Contents

- [Code of Conduct](#code-of-conduct)
- [I don't want to read this whole thing. I've a question.](#i-dont-want-to-read-this-whole-thing-ive-a-question)
- [How Can I Contribute?](#how-can-i-contribute)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Enhancements](#suggesting-enhancements)
- [Your First Code Contribution](#your-first-code-contribution)
- [Merge Requests](#merge-requests)
- [Style Guides](#style-guides)
- [Git Commit Messages Style Guide](#git-commit-messages-style-guide)



## Code of Conduct

This project and everyone who participates in it are governed by the [Code of Conduct](CODE_OF_CONDUCT.md).
By participating, you're expected to uphold this code.
Report unacceptable behavior to [lucas.greuloch@pm.me](mailto:lucas.greuloch@pm.me).



## I don't want to read this whole thing. I've a question.

First, you can read the documentation or search the wiki for the information you need.

If you need help by the developers, create an issue with your question and add the label *support* to it.
*Perform a cursory search* to see if your question has already been asked.
If it has, add a comment to the existing issue instead of opening a new one.

___
*Note:* If you find a *Closed* issue that seems like it is the same topic that you're looking for, open a new issue and include a link to the original issue in the body of your new one.
___

## How Can I Contribute?

### Reporting Bugs

This section guides you through submitting a bug report for Rust-FlightPlan.
Following these guidelines helps maintainers and the community to understand your report, reproduce the behavior, and find related reports.

Before creating bug reports,
check [this list](#before-submitting-a-bug-report) as you might find out that you don't need to create one.
When you're creating a bug report, [include as many details as possible](#how-do-i-submit-a-good-bug-report).
Fill out [the required template](.gitlab/issue_templates/Bug.md),
the information it asks for helps us to resolve issues faster.
You can likewise submit a new (bug-)issue
by sending it to [contact-project+flightsimtools-r-fst-r-fp-55242696-issue-@incoming.gitlab.com](mailto:contact-project+flightsimtools-r-fst-r-fp-55242696-issue-@incoming.gitlab.com)



#### Before Submitting A Bug Report

* *Check the FAQ* for a list of common questions and problems.
* *Perform a cursory search* to see if the problem has already been reported. If it has, *and the issue is still open*, add a comment to the existing issue instead of opening a new one.

___
*Note:* If you find a *Closed* issue that seems like it is the same thing that you're experiencing, open a new issue and include a link to the original issue in the body of your new one.
___

#### How Do I Submit A (Good) Bug Report?

Bugs are tracked as Issues.
Provide the following information by filling in [the template](.gitlab/issue_templates/Bug.md).

Explain the problem and include additional details to help maintainers reproduce the problem:

* *Use a clear and descriptive title* for the issue to identify the problem.
* *Describe the exact steps which reproduce the problem* in as many details as possible. 
For example, start by explaining how you started the software.
* When listing steps, *don't only say what you did, but explain how you did it*.
* *Provide specific examples to demonstrate the steps.*
Include links to files, other projects, or copy/pasteable snippets, which you use in those examples.
If you're providing snippets in the issue,
* Use [Markdown code blocks](https://gitlab.com/help/user/markdown#code-and-syntax-highlighting).
* *Describe the behavior you observed after following the steps* and point out what exactly is the problem with that behavior.
* *Explain which behavior you expected to see instead and why.*
* *Include screenshots and animated GIFs* which show you following the described steps and clearly demonstrate the problem.
* *If you're reporting that Rust-FlightPlan crashed*, include a crash report with a stack trace from the operating system.
  Include the crash report in the issue in a [code block](https://gitlab.com/help/user/markdown#code-and-syntax-highlighting),
  a file attachment,
  or put it in a [snippet](https://gitlab.com/dashboard/snippets) and provide the link to that snippet.
* *If the problem wasn't triggered by a specific action*, describe what you were doing before the problem happened and share more information using the guidelines below.


Provide more context by answering these questions:

* *Did the problem start happening recently,* (for example, after updating to a new version of Rust-FlightPlan), or was this always a problem?
* If the problem started happening recently, *can you reproduce the problem in an older version of Rust-FlightPlan?*
  What is the most recent version in which the problem doesn't happen?
  You can download older versions of Rust-FlightPlan from the release's directory.
* *Can you reliably reproduce the issue?*
  If not, provide details about how often the problem happens and under which conditions it normally happens.
* If the problem is related to working with files (for example, opening and editing files), *does the problem happen for all files and projects or only some?* Is there anything special about the files you're using?


Include details about your configuration and environment:

* *Which version of Rust-FlightPlan are you using?*
* *What is the name and version of the OS you're using?*
* *Are you running Rust-FlightPlan in a virtual machine or a container?*
  If so, which VM/Container software are you using,
  and what operating systems and versions are used for the host and the guest?


### Suggesting Enhancements

This section guides you through submitting an enhancement suggestion for Rust-FlightPlan, including completely new features and minor improvements to existing functionality.
Following these guidelines helps maintainers and the community to understand your suggestion and find related suggestions.

Before creating enhancement suggestions,
check [this list](#before-submitting-an-enhancement-suggestion) as you might find out that you don't need to create one.
When you're creating an enhancement suggestion,
[include as many details as possible](#how-do-i-submit-a-good-enhancement-suggestion).
Fill in [the template](.gitlab/issue_templates/Feature.md),
including the steps that you imagine you would take if the feature you're requesting existed.
You can likewise submit a new (feature-)issue
by sending it to [contact-project+flightsimtools-r-fst-r-fp-55242696-issue-@incoming.gitlab.com](mailto:contact-project+flightsimtools-r-fst-r-fp-55242696-issue-@incoming.gitlab.com).


#### Before Submitting An Enhancement Suggestion

* *Perform a cursory search* to see if the enhancement has already been suggested.
  If it has, add a comment to the existing issue instead of opening a new one.

___
*Note:* If you find a *Closed* issue that seems like it is the same thing that you're suggesting, and you have new arguments to implement it, open a new issue and include a link to the original issue in the body of your new one.
___

#### How Do I Submit A (Good) Enhancement Suggestion?

Enhancement suggestions are tracked as Issues. Create an issue and provide the following information:

* *Use a clear and descriptive title* for the issue to identify the suggestion.
* *Provide a step-by-step description of the suggested enhancement* in as many details as possible.
* *Provide specific examples to demonstrate the steps.*
  Include copy/pasteable snippets which you use in those examples,
* as [Markdown code blocks](https://gitlab.com/help/user/markdown#code-and-syntax-highlighting).
* *Describe the current behavior* and *explain which behavior you expected to see instead* and why.
* *Include screenshots and animated GIFs* which help you demonstrate the steps or point out the part which the suggestion is related to.
* *Explain why this enhancement would be useful* to most Rust-FlightPlan users.
* *Specify which version of Rust-FlightPlan you're using.*
* *Specify the name and version of the OS you're using.*


### Your First Code Contribution

Unsure where to begin contributing to Rust-FlightPlan? You can start by looking through these `beginner` and `help-wanted` issues:

* *Beginner issues* – issues which should only require a few lines of code, and a test or two.
* *Help wanted issues* – issues which should be a bit more involved than `beginner` issues.


### Merge Requests

* Open a merge request (MR) against the _develop_ branch.
* Prefix the MR name with one of the following types: "FEATURE", "BUG", "CHORE", "META" and the affected part of the software.
  For example, [FEATURE – Database] or [BUG – API].
* Fill in the [required template](.gitlab/merge_request_templates/Merge_Request.md).
* Don't include issue numbers in the title.
* Include screenshots or animated GIFs in your merge request whenever possible.
* End all files with a newline.
* Avoid platform-dependent code.


## Style Guides


### Git Commit Messages Style Guide

* Use the present tense ("Add feature" not "Added feature")
* Use the imperative mood ("Move cursor to…" not "Moves cursor to…")
* Limit the first line to 72 characters or fewer
* Reference issues and merge requests (`relates to #XYZ` or `relates to !XYZ`)
* Add `[skip ci]` to the commit message to skip the ci pipeline if it doesn't need to run