#!/usr/bin/env python3
import json
import logging
import random
from pathlib import Path

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


def load_settings():
    """Load settings from settings.json file."""
    settings_file = Path(__file__).parent / "settings.json"
    if settings_file.exists():
        try:
            with open(settings_file, "r") as f:
                settings = json.load(f)

            return settings
        except (json.JSONDecodeError, IOError) as e:
            logger.warning(f"Failed to load settings.json: {e}")
            return {}
    else:
        logger.info("settings.json not found, using default settings")
        return {}


def configure_debug_logging(settings):
    """Configure debug logging to file if debug is enabled in settings."""
    if settings.get("debug", False):
        # Create log directory if it doesn't exist
        log_dir = Path("log")
        log_dir.mkdir(exist_ok=True)

        # Add file handler for debug logs
        debug_handler = logging.FileHandler(log_dir / "debug.log")
        debug_handler.setLevel(logging.DEBUG)
        debug_formatter = logging.Formatter(
            "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
        )
        debug_handler.setFormatter(debug_formatter)

        # Set root logger to DEBUG level and add the file handler
        root_logger = logging.getLogger()
        root_logger.setLevel(logging.DEBUG)
        root_logger.addHandler(debug_handler)

        logger.info("Debug logging enabled, writing to log/debug.log")
