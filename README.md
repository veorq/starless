# Starless

Advanced AI-based **supply-chain** security intelligence for Go projects.

```
starless <go.mod file> <max stars> <config file>
```

It shows GitHub repos with less than `max stars` and shows the date of
the last commit.

You'll need a GitHub API token (public permissions sufficient) to
avoid the API rate limiting, see
[config.json_example](./config.json_example).

The viability of the stars count as a popularity metric is studied in
papers such as [Understanding the Factors that Impact the
Popularity of GitHub Repositories](https://arxiv.org/pdf/1606.04984)
(2016).
