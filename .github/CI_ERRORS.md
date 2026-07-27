# CI/CD Workflow Errors

This file documents known CI/CD workflow errors and issues discovered during automated scanning.

**Last Updated**: 2026-07-27

## Quick Links
- Full Error Report: See main [CI/CD Error Report](../../CI_ERRORS_REPORT.md)
- GitHub Actions Documentation: https://docs.github.com/en/actions
- Troubleshooting Guide: [TROUBLESHOOTING.md](.github/TROUBLESHOOTING.md)

## How to Check Workflow Status

```bash
# List recent workflow runs
gh run list -R Mullassery/$REPO_NAME --limit 10

# View details of a specific run
gh run view <RUN_ID> -R Mullassery/$REPO_NAME

# View logs from a failed run
gh run view <RUN_ID> -R Mullassery/$REPO_NAME --log-failed
```

## Known Issues in This Repository

Please see the main CI/CD Error Report for details specific to this repository:
[CI_ERRORS_REPORT.md](../../CI_ERRORS_REPORT.md#$repo)

## Action Items

- Review the full error report
- Check your GitHub Actions workflows in `.github/workflows/`
- Update deprecated action versions
- Test fixes in a development branch before merging

## Need Help?

- Check the [GitHub Actions Documentation](https://docs.github.com/en/actions)
- Review the main error report for details on your specific issues
- Run `gh run view <run-id> --log` to see full logs

