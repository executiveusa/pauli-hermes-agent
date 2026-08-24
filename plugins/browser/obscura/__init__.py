"""Obscura local browser plugin — bundled, auto-loaded.

Mirrors the ``plugins/browser/<vendor>/`` layout used by ``browser_use``,
``browserbase``, and ``firecrawl``: ``provider.py`` holds the provider class;
``__init__.py::register`` instantiates and registers it.

Vendored from https://github.com/SGavrl/hermes-plugin-obscura (Apache-2.0),
which wraps https://github.com/h4ckf0r0day/obscura — a Rust headless browser
that speaks the Chrome DevTools Protocol with no Chrome or Node.js
dependency. Opt-in via ``browser.cloud_provider: obscura``; the registry
never auto-selects it, same as every other browser provider here.
"""

from __future__ import annotations

from plugins.browser.obscura.provider import ObscuraBrowserProvider


def register(ctx) -> None:
    """Register the Obscura provider with the plugin context."""
    ctx.register_browser_provider(ObscuraBrowserProvider())
