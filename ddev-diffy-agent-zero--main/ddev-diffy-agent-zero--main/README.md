# DDEV Diffy

## What is Diffy?

Diffy is a visual regression testing service https://diffy.website. It is primary tailored to testing Drupal and WordPress websites.

This repository is a recipe for DDEV to have a local worker that will allow taking screenshots from local website and upload them to Diffy for further comparisons.

## Installation

For DDEV v1.23.5 or above run

```sh
ddev add-on get diffywebsite/ddev-diffy
```

For earlier versions of DDEV run

```sh
ddev get diffywebsite/ddev-diffy
```

Register an account in Diffy and create an [API key](https://app.diffy.website/#/keys).

Once you have your container ready after `ddev restart` we need to provide API key and project ID to `.env` file. For that go to `.ddev/diffy-worker`, copy existing example file `cp .env.example .env` and edit `.env` file to provide your credentials.

To run the screenshots do `ddev screenshot`. It will produce a URL to your screenshots uploaded to Diffy. Next you can compare them to your any other sets (production, staging, baseline etc.).

Remember to check our [documentation page](https://docs.diffy.website/features/local-development/ddev-add-on). 

**Contributed and maintained by [@ygerasimov](https://github.com/ygerasimov)**
