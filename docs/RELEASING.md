# Releasing PixelPulse

## 1. Verify the candidate while the repository is private

1. Verify that the release candidate branch is on GitHub. The [Microsoft Store submission guide](https://github.com/renahangil/pixelpulse/blob/efafcf3717ebe576e87bbec87a8540cc1fc684fd/docs/MICROSOFT-STORE.md) is available at its historical commit. Use the current release version when packaging.
2. Review the candidate's diff against remote `main`. Review all Git branches, tags and history, plus Actions logs and artifacts, releases, issues and repository attachments for credentials, machine-specific diagnostics, personal information and private publisher records. Rotating any exposed credential comes before publication. Removing a file from the release branch leaves earlier Git history intact.
3. In **Security**, review Dependabot alerts against both the existing default branch and the proposed lockfiles. A newer dependency tree can resolve an alert once merged, but verify the actual advisory and affected version. Review dependency licenses and asset notices, including GPL-3.0 and Silkscreen OFL.
4. In **Settings → Actions → General**, enable GitHub Actions and choose read-only default workflow token permissions. Keep workflow permission to create or approve pull requests off. The supplied CI requires only read access to repository contents. Check any repository-level action restrictions against the two SHA-pinned GitHub actions.

[Repository visibility implications](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/managing-repository-settings/setting-repository-visibility) · [Actions settings](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository)

## 2. Open and protect the release PR

1. Open or update the release PR into `main`. Confirm the proposed diff and review the **Windows checks** job under **Checks**. CI runs formatting, Clippy, tests and the release build on Windows. It also runs on the subsequent push to `main`.
2. In **Settings → Rules → Rulesets → pr-general-ruleset**, verify enforcement is **Active**, the target includes `main`, and pull requests are required. Enable **Require status checks to pass**, selecting the exact check displayed by GitHub after its first successful run. Block force pushes and deletion for `main`. Review bypass actors and keep them limited. Enable up-to-date branches if you want the check to run against the latest base.
3. When a second maintainer can review, require one independent approval and merge after approval and green checks. If you maintain the repository alone, use zero required approvals until a reviewer is available. Authors cannot approve their own PRs. Ruleset availability for private personal repositories depends on the account plan.
4. Merge the approved PR. A normal merge commit preserves the sequence of phase commits on `main`. In **Actions → CI**, open the run for the resulting `main` commit and confirm **Windows checks** passes.

If you need command-line access, authenticate through the browser and complete SSO if GitHub requests it:

```powershell
gh auth login --web
gh api repos/renahangil/pixelpulse/rulesets/27759713
$prNumber = Read-Host 'Pull request number'
gh pr checks $prNumber --repo renahangil/pixelpulse
```

[Rules for protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets) · [Reviewing pull requests](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/reviewing-changes-in-pull-requests/about-pull-request-reviews)

## 3. Configure and make the repository public

1. Wait for the merged `main` commit to pass CI and for the audit to be complete. Confirm the candidate produces the expected executable and the default branch shows the final README, license and design.
2. On the repository home page, use the **About** gear. Suggested description: **A lightweight pixel-art system monitor for Windows with a customizable floating island.** Suggested topics: `windows`, `system-monitor`, `pixel-art`, `tauri`, `rust` and `typescript`. Set the website to the actual Store page when it is live.
3. In **Settings → Security** or **Security and quality**, check Dependabot alerts, secret scanning and push protection as available for your plan and visibility. Enable private vulnerability reporting for the public repository if you can monitor it. Use two-factor authentication on the owner account and review collaborators, tokens and GitHub App access.
4. In **Settings → General → Danger Zone → Change repository visibility**, choose **Public** and confirm. Recheck rulesets and security controls after the change. Open the repository in a signed-out browser and verify the README, license, issues and archive access.

[GitHub release visibility](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/managing-repository-settings/setting-repository-visibility) · [Private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/working-with-repository-security-advisories/configuring-private-vulnerability-reporting-for-a-repository)

## 4. Publish the 1.1.0.0 release

After the PR merges, the CI workflow creates `1.1.0.0` on the tested `main` commit once **Windows checks** succeeds. The **Create release tag** job is only eligible on pushes to `main` and leaves any existing tag in place. Check the job outcome and verify the tag points to the intended green commit. If tag creation is restricted, review the applicable ruleset before retrying the job.

Build from the tag in a clean checkout. `git rev-parse HEAD` should report the same SHA as the green run.

```powershell
git fetch origin tag 1.1.0.0
git switch --detach 1.1.0.0
git status --porcelain
git rev-parse HEAD
$env:PIXELPULSE_RELEASE_DATE = (Read-Host 'Release date (YYYY-MM-DD)')
npm ci
npm run tauri build -- --no-bundle
Get-FileHash -Algorithm SHA256 .\src-tauri\target\release\pixelpulse.exe
```

In **Releases → Draft a new release**, select tag `1.1.0.0` and use the title **PixelPulse 1.1.0.0**. Summarize island controls, metrics, diagnostics, languages and startup. State Windows 11 x64 and WebView2 requirements, note platform-dependent GPU sensors, and identify the supported distribution channel. If you attach a standalone executable, use the tested x64 artifact from that revision and include its SHA-256. Ensure the matching GPL-3.0 source, instructions and notices are available. Preview the release while editing and verify downloads after publication.

For the Store, build with the registered package identity and follow the separate Partner Center checklist. Keep the upload package, hash, source revision and certification report. Verify that the Store listing is publicly available before announcing Microsoft Store downloads. [Managing GitHub releases](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)
