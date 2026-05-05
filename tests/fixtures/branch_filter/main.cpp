// Feature flag system
class FeatureFlags {
public:
    bool enableLogging;
    bool enableCache;
    bool enableSSL;
};

FeatureFlags getFlags() {
    FeatureFlags f;
    f.enableLogging = true;
    f.enableCache = false;
    f.enableSSL = true;
    return f;
}
