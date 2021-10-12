# `git-pr`
*Pull requests without Git{Hub,Lab,Whatever}*

[License](https://github.com/robertdfrench/portunusd/blob/trunk/LICENSE)
&VerticalSeparator;
[Roadmap](https://github.com/robertdfrench/git-pr/milestones)
&VerticalSeparator;
[Download](https://github.com/robertdfrench/git-pr/releases)

Git does not support pull requests by default, but it could! By following a few
simple conventions, any shared repository can support pull requests without
additional tooling required from the host. `git-pr` is tooling to implement
those conventions, making it easy to create, approve, or destroy pull requests
-- all from the command line:

```console
# Create a new branch for an emergency hotfix
$ git pr-create hotfix
Switched to a new branch 'hotfix/0'
...
 * [new branch]      hotfix/0 -> hotfix/0
Branch 'hotfix/0' set up to track remote branch 'hotfix/0' from 'origin'.
# (code code code...)
# Submit for review... it's just a branch!
$ git push
```

Meanwhile, your collaborators can check for the latest updates, review your
changes, and even merge them:

```console
$ git pr-list
...
 * [new branch]      hotfix/0 -> hotfix/0
hotfix
remove-hardcoded-passwords
use-git-pr-tool
# (review review review...)
$ git pr-accept hotfix
```

Given this minimalist approach, the means of communication is up to you. We
don't try to force you into some mechanism that just ends up emailing you
anyway. :smirk:
