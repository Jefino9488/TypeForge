#ifndef TYPEFORGE_ENGINE_H
#define TYPEFORGE_ENGINE_H

#include <fcitx/addonfactory.h>
#include <fcitx/addoninstance.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/candidatelist.h>
#include <fcitx/instance.h>
#include <memory>
#include <string>
#include <vector>

extern "C" {
    struct C_Prediction {
        const char* text;
        float score;
        uint32_t source;
    };

    struct C_PredictionList {
        C_Prediction* predictions;
        size_t count;
        uint64_t generation;
    };

    typedef void (*PredictCallback)(C_PredictionList* list, void* user_data);

    void typeforge_predict_async(const char* prefix, uint64_t generation, PredictCallback callback, void* user_data);
    C_PredictionList* typeforge_predict_sync(const char* prefix);
    void typeforge_free_prediction_list(C_PredictionList* list);
}

class TypeForgeEngine : public fcitx::InputMethodEngineV2 {
public:
    explicit TypeForgeEngine(fcitx::Instance* instance);
    ~TypeForgeEngine() override;

    void keyEvent(const fcitx::InputMethodEntry& entry, fcitx::KeyEvent& keyEvent) override;
    void reset(const fcitx::InputMethodEntry&, fcitx::InputContextEvent& event) override;

    void commitString(fcitx::InputContext* ic, const std::string& str);
    
    static void onPredictionsReady(C_PredictionList* list, void* user_data);
    
    fcitx::Instance* instance() const { return instance_; }
    fcitx::InputContext* activeContext() const { return active_ic_; }
    uint64_t currentGeneration() const { return current_generation_; }

private:
    void updatePreedit(fcitx::InputContext* ic);

    fcitx::Instance* instance_;
    fcitx::InputContext* active_ic_ = nullptr;
    std::string preedit_;
    uint64_t current_generation_ = 0;
};

class TypeForgeEngineFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance* create(fcitx::AddonManager* manager) override;
};

class TypeForgeCandidateWord : public fcitx::CandidateWord {
public:
    TypeForgeCandidateWord(std::string text, fcitx::InputContext* ic, TypeForgeEngine* engine);
    void select(fcitx::InputContext* ic) const override;
private:
    std::string text_;
    fcitx::InputContext* ic_;
    TypeForgeEngine* engine_;
};

#endif // TYPEFORGE_ENGINE_H
