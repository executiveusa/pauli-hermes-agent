"""Canonical execution backend router for Hermes.

Terabithia owns mission identity and policy. Hermes owns execution routing.
Backends execute work and must report isolation truthfully.
"""

from .router import (
    BackendCapabilities,
    BackendUnavailable,
    CapabilityManifest,
    ExecutionBackend,
    ExecutionRequest,
    ExecutionResult,
    ExecutionRouter,
    IsolationClass,
    RequiredIsolationUnavailable,
)

__all__ = [
    "BackendCapabilities",
    "BackendUnavailable",
    "CapabilityManifest",
    "ExecutionBackend",
    "ExecutionRequest",
    "ExecutionResult",
    "ExecutionRouter",
    "IsolationClass",
    "RequiredIsolationUnavailable",
]
