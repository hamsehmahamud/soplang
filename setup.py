#!/usr/bin/env python3
import os
import sys

from setuptools import find_packages, setup

from psrc.core.version import VERSION

# Read requirements from psrc (Python implementation lives in psrc/)
with open(os.path.join(os.path.dirname(__file__), "psrc", "requirements.txt")) as f:
    requirements = [
        line.strip() for line in f if not line.startswith("#") and line.strip()
    ]

# Filter out readline on Windows, as it's not compatible
if sys.platform == "win32":
    requirements = [req for req in requirements if not req.startswith("readline")]

# Windows-specific requirements
windows_requirements = [
    "pypiwin32",
    "pywin32",
]

setup(
    name="soplang",
    version=VERSION,
    description="The Somali Programming Language",
    author="Sharafdin",
    author_email="info@soplang.org",
    url="https://www.soplang.org/",
    packages=find_packages(),
    include_package_data=True,
    install_requires=requirements,
    extras_require={
        "windows": windows_requirements,
    },
    entry_points={
        "console_scripts": [
            "soplang=psrc.__main__:main",
        ],
    },
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Environment :: Console",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Topic :: Software Development :: Interpreters",
    ],
    python_requires=">=3.6",
)
