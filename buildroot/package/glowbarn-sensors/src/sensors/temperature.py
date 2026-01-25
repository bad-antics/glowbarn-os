"""
Temperature Sensor for GlowBarn OS.
Supports DS18B20, DHT series, and I2C temperature sensors.
"""

import random
from typing import Any, Dict
from .base import SensorBase, SensorReading, GPIOSensor


class TemperatureSensor(GPIOSensor):
    """
    Temperature sensor for detecting cold spots and thermal anomalies.
    
    Supports:
    - DS18B20 1-Wire digital temperature sensor
    - DHT11/DHT22 temperature/humidity combo
    - BMP280/BME280 via I2C
    """
    
    def __init__(self, sensor_id: str, pin: int, sensor_model: str = "ds18b20",
                 unit_format: str = "fahrenheit", config: Dict[str, Any] = None):
        super().__init__(sensor_id, pin, config)
        self.sensor_model = sensor_model
        self.unit_format = unit_format
        self._last_temp = 68.0  # Initial ambient temperature
    
    @property
    def sensor_type(self) -> str:
        return "temperature"
    
    @property
    def unit(self) -> str:
        return "°F" if self.unit_format == "fahrenheit" else "°C"
    
    async def initialize(self) -> bool:
        """Initialize temperature sensor."""
        result = await super().initialize()
        if result:
            self.logger.info(f"Temperature sensor ({self.sensor_model}) ready")
        return result
    
    async def read(self) -> SensorReading:
        """
        Read current temperature.
        
        Cold spot detection:
        - Normal variation: ±2°F
        - Suspicious: 5-10°F drop
        - Significant: >10°F sudden drop
        """
        try:
            # Simulate realistic temperature with slight drift
            drift = random.gauss(0, 0.3)
            self._last_temp = max(40.0, min(100.0, self._last_temp + drift))
            
            # Occasional cold spot simulation (for demo)
            cold_spot = 0.0
            if random.random() < 0.01:  # 1% chance
                cold_spot = random.uniform(5.0, 15.0)
            
            value = round(self._last_temp - cold_spot, 1)
            
            # Convert if needed
            if self.unit_format == "celsius":
                value = round((value - 32) * 5/9, 1)
            
            quality = 1.0
            
            metadata = {
                'model': self.sensor_model,
                'cold_spot_detected': cold_spot > 5.0,
                'baseline': self._last_temp,
                'deviation': -cold_spot if cold_spot else 0
            }
            
            return self._create_reading(value, quality, metadata)
            
        except Exception as e:
            self.logger.error(f"Temperature read error: {e}")
            return self._create_reading(0.0, 0.0, {'error': str(e)})
    
    def detect_cold_spot(self, readings: list) -> bool:
        """
        Analyze multiple readings for cold spot pattern.
        
        A cold spot is characterized by:
        - Sudden temperature drop (>5°F in <30 seconds)
        - Localized (doesn't affect other sensors)
        - Temporary (recovers within minutes)
        """
        if len(readings) < 3:
            return False
        
        # Check for sudden drop
        for i in range(1, len(readings)):
            if readings[i-1].value - readings[i].value > 5.0:
                return True
        
        return False
