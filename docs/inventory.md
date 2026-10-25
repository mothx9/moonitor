# Host inventory

The application owns the local inventory in its versioned `settings.sqlite`
registry. Open **Setup → Hosts & Connections** to add, edit or remove hosts.
First run enrolls the local computer with a generated stable ID and real collectors.

This repository contains no personal fleet inventory. Test identities are explicit
fixtures compiled only in tests; public screenshots are labeled synthetic examples.
Legacy `fleet.toml` files are imported once without changing node IDs or the original
file. Removing a host preserves its history. Private user config/databases stay
outside Git. History cleanup and repository visibility are separate release gates.
