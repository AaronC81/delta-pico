#pragma once

class Application {
public:
    virtual void init() = 0;
    virtual void tick() = 0;
};
