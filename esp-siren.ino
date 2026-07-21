#include <Arduino.h>

namespace {
constexpr uint8_t kSpeakerPin = 25;
#ifdef LED_BUILTIN
constexpr uint8_t kLedPin = LED_BUILTIN;
#else
// GPIO2 is the onboard user LED on common ESP-WROOM-32 DevKit boards.
constexpr uint8_t kLedPin = 2;
#endif
constexpr uint16_t kHighToneHz = 900;
constexpr uint16_t kLowToneHz = 600;
constexpr uint16_t kToneDurationMs = 450;
}

void setup() {
  pinMode(kSpeakerPin, OUTPUT);
  pinMode(kLedPin, OUTPUT);
  noTone(kSpeakerPin);
  digitalWrite(kLedPin, LOW);
}

void loop() {
  // The alternating tones produce the familiar "wee-woo" police siren.
  tone(kSpeakerPin, kHighToneHz);
  digitalWrite(kLedPin, HIGH);
  delay(kToneDurationMs);

  tone(kSpeakerPin, kLowToneHz);
  digitalWrite(kLedPin, LOW);
  delay(kToneDurationMs);
}
