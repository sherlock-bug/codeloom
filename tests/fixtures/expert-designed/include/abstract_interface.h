#ifndef ABSTRACT_INTERFACE_H
#define ABSTRACT_INTERFACE_H

// Abstract interface — pure virtual method, called via pointer.
// Reproduces the leveldb Env::GetChildren pattern:
//   env_->GetChildren(dir, &children)
// where Env is abstract and env_ is Env*.

class Interface {
public:
    virtual ~Interface() = default;
    virtual int Compute(int value) const = 0;   // pure virtual
    virtual int Process(int value) const = 0;   // pure virtual (for comparison)
};

class Impl : public Interface {
public:
    int Compute(int value) const override;
    int Process(int value) const override;
};

#endif
