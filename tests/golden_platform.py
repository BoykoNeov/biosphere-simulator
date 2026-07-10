"""Platform gate for the hex-float regression goldens.

The committed regression goldens are byte-exact only *within a single build on the
platform that generated them* — Windows/UCRT here (determinism invariant #7 is
same-build). A scenario whose evaluation graph touches a transcendental
(``exp`` / ``sqrt`` / ``sin`` / ``**``) lands last-ULP-different on a different libm
(Linux/glibc, macOS), so a byte-exact (or exact ``State``-equality) compare of a
*re-run* against the Windows golden fails off the generation platform — physically
meaningless noise, not a regression. Cross-libm parity for those scenarios is gated
instead by ``tests/crossport/`` with *measured tolerance bands*
(see ``docs/native-port-reference.md``).

So the transcendental regression goldens carry ``windows_golden_only``: their
byte-exactness is asserted only on the generation platform. The gate is by
transcendental *classification*, not by the set that happens to diverge on one
glibc build — a few contracting/regulator-erased finals (thermal, greenhouse,
lighting, the 5-yr consumer chamber) coincidentally match on some libms, but relying
on that would make CI brittle across glibc versions. Pure-arithmetic goldens
(crew / eclss / cabin_gas / water_recovery / the basic-arithmetic demo) are bit-exact
on every conformant platform and stay ungated.
"""

import sys

import pytest

windows_golden_only = pytest.mark.skipif(
    sys.platform != "win32",
    reason=(
        "hex-float regression golden is byte-exact only on its Windows/UCRT "
        "generation platform; cross-libm parity is gated by tests/crossport "
        "(measured tolerance bands, docs/native-port-reference.md)"
    ),
)
