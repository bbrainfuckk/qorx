# Security Model

The public Qorx repository is a documentation and release surface. It is not the private implementation repository.

## What Public Documentation Can Protect

Public docs can protect Qorx by being precise:

- explain the product without claiming the private core is open;
- document benchmark methodology without publishing private materials;
- publish release checksums and signatures;
- describe safe issue-reporting practices;
- make the Qorx Void boundary easy to verify.

## What Public Documentation Cannot Protect

Public docs cannot stop reverse engineering of a public binary. A public binary can be inspected, patched, and studied.

The protection strategy is therefore not secrecy by wording. The strategy is to ship only a narrow public artifact and keep private product material out of the public package.

## Release Review Questions

Before publishing a public release, answer these questions:

- Does the package contain only approved files?
- Does every byte in the package remain safe if inspected?
- Are public bundle hashes pinned and checked?
- Does the command fail closed on unsupported hardware?
- Are raw prompts and model outputs excluded from generated artifacts?
- Are private paths, usernames, hostnames, and credentials absent?
- Does the wording avoid unsupported claims?

If the answer is not yes, do not publish.
