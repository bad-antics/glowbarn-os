"""
Motion Sensor for GlowBarn OS.
PIR-based motion detection for paranormal investigation.
"""

import random
from typing import Any, Dict
from .base import GPIOSensor, SensorReading


class MotionSensor(GPIOSensor):
    """
    PIR Motion sensor for detecting movement.
    
    Can detect:
    - Human-sized movement
    - Heat signature changes
    - Infrared anomalies
    """
    
    def __init__(self, sensor_id: str, pin: int, debounce_ms: int = 100,
                 config: Dict[str, Any] = None):
        super().__init__(sensor_id, pin, config)
        self.debounce_ms = debounce_ms
        self._motion_count = 0
        self._last_motion_time = None
    
    @property
    def sensor_type(self) -> str:
        return "motion"
    
    @property
    def unit(self) -> str:
        return "bool"
    
    async def read(self) -> SensorReading:
        """Read motion detection state."""
        try:
            # Simulate motion detection (1% chance)
            detected = random.random() < 0.01
            
            if detected:
                self._motion_count += 1
                from datetime import datetime
                self._last_motion_time = datetime.now()
            
            metadata = {
                'total_events': self._motion_count,
                'last_motion': self._last_motion_time.isoformat() if self._last_motion_time else None
            }
            
            return self._create_reading(detected, 1.0, metadata)
            
        except Exception as e:
            self.logger.error(f"Motion read error: {e}")
            return self._create_reading(False, 0.0, {'error': str(e)})
