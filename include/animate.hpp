#pragma once

#include <string.h>

enum class Easing {
    LINEAR,
    EASE_OUT,
};

template<int D>
class Animate {
public:
    const unsigned int DIMENSIONS = D;

    Animate(
        int _startValue[D], int _targetValue[D], unsigned int _timeFrame,
        Easing _easing = Easing::LINEAR
    ) : timeFrame(_timeFrame), timeElapsed(0), easing(_easing)
    {
        memcpy(startValue, _startValue, sizeof(int) * D);
        memcpy(currentValue, _startValue, sizeof(int) * D);
        memcpy(targetValue, _targetValue, sizeof(int) * D);

        for (int i = 0; i < D; i++) {
            currentFloatValue[i] = (float)currentValue[i];
        }
    }

    bool tick(void) {
        if (timeElapsed < timeFrame) {
            float thisStep[D];
            step(thisStep); 
            Serial.print("Step: ");
            Serial.print(thisStep[0]);
            Serial.print(" ");
            Serial.println(thisStep[1]);
            for (int i = 0; i < D; i++) {
                currentFloatValue[i] += thisStep[i];
            }
            timeElapsed++;

            copyCurrentValueFloatToInt();
            return true;
        } else if (timeElapsed == timeFrame) {
            for (int i = 0; i < D; i++) {
                currentFloatValue[i] = targetValue[i];
            }
            timeElapsed++;

            copyCurrentValueFloatToInt();
            return true;
        } else {
            return false;
        }
    }

    void step(float step[D]) {
        switch (easing) {
        case Easing::LINEAR:
            for (int i = 0; i < D; i++) {
                step[i] = (float)(targetValue[i] - startValue[i]) / (float)timeFrame;
            }
            break;
        case Easing::EASE_OUT:
            // Quint ease: https://easings.net/#easeOutQuint
            float timeStepRatio = 1.0 / timeFrame;
            
            float pointNow = 1 - pow(1 - timeStepRatio * timeElapsed, 5);
            float pointThen = 
                timeElapsed == 0
                ? pointNow
                : 1 - pow(1 - timeStepRatio * (timeElapsed - 1), 5);
            float delta = pointNow - pointThen;

            for (int i = 0; i < D; i++) {
                step[i] = (float)(targetValue[i] - startValue[i]) * delta;
            }
            break;
        }
    }

    int currentValue[D];
protected:
    int startValue[D];
    int targetValue[D];
    unsigned int timeFrame;
    Easing easing;

    float currentFloatValue[D];
    unsigned int timeElapsed;

    void copyCurrentValueFloatToInt(void) {
        for (int i = 0; i < D; i++) {
            currentValue[i] = (int)currentFloatValue[i];
        }
    }
};
