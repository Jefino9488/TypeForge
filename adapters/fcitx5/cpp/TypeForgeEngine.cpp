#include "TypeForgeEngine.h"
#include <fcitx/candidatelist.h>
#include <fcitx/event.h>
#include <fcitx/inputpanel.h>
#include <fcitx/userinterfacemanager.h>
#include <fcitx/inputcontextproperty.h>
#include <fcitx-utils/event.h>
#include <fcitx-utils/keysym.h>
#include <fcitx-utils/keysymgen.h>
#include <fcitx-utils/log.h>
#include <fcitx-utils/textformatflags.h>
#include <fcitx/instance.h>
#include <fcitx/addonmanager.h>
#include <unistd.h>
#include <fcntl.h>

FCITX_ADDON_FACTORY(TypeForgeEngineFactory)

fcitx::AddonInstance* TypeForgeEngineFactory::create(fcitx::AddonManager* manager) {
    return new TypeForgeEngine(manager->instance());
}

// ---------------------------------------------
// TypeForgeEngine
// ---------------------------------------------

TypeForgeEngine::TypeForgeEngine(fcitx::Instance* instance) 
    : instance_(instance) {
    if (pipe(pipe_fd_) == 0) {
        fcntl(pipe_fd_[0], F_SETFL, O_NONBLOCK);
        fcntl(pipe_fd_[1], F_SETFL, O_NONBLOCK);
        
        io_event_ = instance->eventLoop().addIOEvent(pipe_fd_[0], fcitx::IOEventFlag::In, [this](fcitx::EventSourceIO*, int, fcitx::IOEventFlags) {
            char buf[128];
            while (read(pipe_fd_[0], buf, sizeof(buf)) > 0) {}
            this->processResultQueue();
            return true;
        });
    }
}

TypeForgeEngine::~TypeForgeEngine() {
    io_event_.reset();
    close(pipe_fd_[0]);
    close(pipe_fd_[1]);
}

void TypeForgeEngine::reset(const fcitx::InputMethodEntry&, fcitx::InputContextEvent& event) {
    preedit_.clear();
    current_generation_++;
    active_ic_ = nullptr;
    suggestion_explicitly_selected_ = false;
    event.inputContext()->inputPanel().reset();
    event.inputContext()->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
    event.inputContext()->updatePreedit();
}

void TypeForgeEngine::keyEvent(const fcitx::InputMethodEntry&, fcitx::KeyEvent& keyEvent) {
    auto ic = keyEvent.inputContext();
    auto key = keyEvent.key();
    
    active_ic_ = ic; // Store current context

    bool hasCtrl = static_cast<bool>(key.states() & fcitx::KeyState::Ctrl);
    bool hasAlt = static_cast<bool>(key.states() & fcitx::KeyState::Alt);

    if (keyEvent.isRelease() || hasCtrl || hasAlt) {
        return;
    }

    if (key.isSimple() && key.sym() >= FcitxKey_a && key.sym() <= FcitxKey_z) {
        preedit_ += static_cast<char>(key.sym());
        suggestion_explicitly_selected_ = false;
        updatePreedit(ic);
        
        current_generation_++;
        std::string app = ic->program();
        std::string surrounding = ic->surroundingText().text();
        FCITX_INFO() << "Requesting prediction for: " << preedit_ << " surrounding: " << surrounding << " generation " << current_generation_;
        typeforge_predict_async(preedit_.c_str(), surrounding.c_str(), app.empty() ? nullptr : app.c_str(), current_generation_, TypeForgeEngine::onPredictionsReady, this);
        keyEvent.filterAndAccept();
        return;
    }

    if (!preedit_.empty()) {
        auto candidateList = ic->inputPanel().candidateList();

        if (key.sym() == FcitxKey_BackSpace) {
            if (!preedit_.empty()) {
                while (!preedit_.empty()) {
                    char c = preedit_.back();
                    preedit_.pop_back();
                    if ((c & 0xC0) != 0x80) {
                        break;
                    }
                }
            }
            suggestion_explicitly_selected_ = false;
            updatePreedit(ic);
            if (!preedit_.empty()) {
                current_generation_++;
                std::string app = ic->program();
                std::string surrounding = ic->surroundingText().text();
                typeforge_predict_async(preedit_.c_str(), surrounding.c_str(), app.empty() ? nullptr : app.c_str(), current_generation_, TypeForgeEngine::onPredictionsReady, this);
            } else {
                current_generation_++;
                ic->inputPanel().reset();
                ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
            }
            keyEvent.filterAndAccept();
            return;
        }

        if (key.sym() == FcitxKey_Escape) {
            preedit_.clear();
            current_generation_++;
            updatePreedit(ic);
            ic->inputPanel().reset();
            ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
            keyEvent.filterAndAccept();
            return;
        }

        if (key.sym() == FcitxKey_Tab || key.sym() == FcitxKey_ISO_Left_Tab) {
            if (candidateList && candidateList->size() > 0) {
                auto cursorMovable = candidateList->toCursorMovable();
                if (cursorMovable) {
                    if (key.sym() == FcitxKey_ISO_Left_Tab || (key.states() & fcitx::KeyState::Shift)) {
                        cursorMovable->prevCandidate();
                    } else {
                        cursorMovable->nextCandidate();
                    }
                    suggestion_explicitly_selected_ = true;
                    ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
                }
            }
            keyEvent.filterAndAccept();
            return;
        }

        if (key.sym() >= FcitxKey_1 && key.sym() <= FcitxKey_9) {
            int index = key.sym() - FcitxKey_1;
            if (candidateList && index < candidateList->size()) {
                std::string text = candidateList->candidate(index).text().toString();
                commitString(ic, text, true);
            } else {
                commitString(ic, preedit_, false);
                ic->commitString(std::string(1, static_cast<char>(key.sym())));
            }
            keyEvent.filterAndAccept();
            return;
        }

        if (key.sym() == FcitxKey_Return || key.sym() == FcitxKey_space) {
            std::string text_to_commit = preedit_;
            bool is_accepted = false;
            
            bool should_commit_candidate = false;
            if (key.sym() == FcitxKey_space) {
                should_commit_candidate = true;
            } else if (key.sym() == FcitxKey_Return && suggestion_explicitly_selected_) {
                should_commit_candidate = true;
            }

            if (should_commit_candidate) {
                if (candidateList && candidateList->size() > 0) {
                    int cursor = candidateList->cursorIndex();
                    if (cursor < 0 || cursor >= candidateList->size()) {
                        cursor = 0;
                    }
                    text_to_commit = candidateList->candidate(cursor).text().toString();
                    is_accepted = true;
                } else {
                    // Fast typing fallback: query synchronously before committing
                    std::string app = ic->program();
                    std::string surrounding = ic->surroundingText().text();
                    C_PredictionList* list = typeforge_predict_sync(preedit_.c_str(), surrounding.c_str(), app.empty() ? nullptr : app.c_str());
                    if (list && list->count > 0 && list->predictions[0].text) {
                        text_to_commit = std::string(list->predictions[0].text);
                        is_accepted = true;
                    }
                    typeforge_free_prediction_list(list);
                }
            }
            
            commitString(ic, text_to_commit, is_accepted);
            
            if (key.sym() == FcitxKey_space) {
                ic->commitString(" ");
            }
            keyEvent.filterAndAccept();
            return;
        }

        // For any other key (punctuation, symbols, uppercase letters), commit the current word and let the key pass through
        if (key.isSimple()) {
            commitString(ic, preedit_, false);
            // Don't filter and accept, so the punctuation passes to the app!
            return;
        }
    }
}

void TypeForgeEngine::updatePreedit(fcitx::InputContext* ic) {
    if (preedit_.empty()) {
        ic->inputPanel().setClientPreedit(fcitx::Text());
        ic->inputPanel().setPreedit(fcitx::Text());
    } else {
        fcitx::Text preeditFormat(preedit_, fcitx::TextFormatFlag::Underline);
        preeditFormat.setCursor(preedit_.size());
        ic->inputPanel().setClientPreedit(preeditFormat);
        ic->inputPanel().setPreedit(preeditFormat);
    }
    ic->updatePreedit();
}

void TypeForgeEngine::commitString(fcitx::InputContext* ic, const std::string& str, bool is_accepted) {
    FCITX_INFO() << "Committing string: '" << str << "'";
    
    // Dispatch learning event
    std::string app = ic->program();
    int64_t delta = is_accepted ? 1 : -1;
    typeforge_learn(str.c_str(), delta, app.empty() ? nullptr : app.c_str());

    preedit_.clear();
    current_generation_++;
    suggestion_explicitly_selected_ = false;
    ic->inputPanel().reset();
    ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
    ic->commitString(str);
}

void TypeForgeEngine::processResultQueue() {
    std::vector<C_PredictionList*> queue;
    {
        std::lock_guard<std::mutex> lock(queue_mutex_);
        queue.swap(result_queue_);
    }

    for (auto list : queue) {
        if (list && list->generation < currentGeneration()) {
            typeforge_free_prediction_list(list);
            continue;
        }

        auto ic = activeContext();
        FCITX_INFO() << "Predictions ready, count: " << (list ? list->count : 0) << ", activeContext: " << (ic != nullptr);
        if (ic && list && list->count > 0) {
            auto candidateList = std::make_unique<fcitx::CommonCandidateList>();
            candidateList->setLayoutHint(fcitx::CandidateLayoutHint::Vertical);
            
            for (size_t i = 0; i < list->count; ++i) {
                if (list->predictions[i].text) {
                    std::string txt(list->predictions[i].text);
                    candidateList->append(std::make_unique<TypeForgeCandidateWord>(txt, ic, this));
                }
            }

            ic->inputPanel().setCandidateList(std::move(candidateList));
            ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
        } else if (ic) {
            ic->inputPanel().setCandidateList(nullptr);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
        }

        typeforge_free_prediction_list(list);
    }
}

void TypeForgeEngine::onPredictionsReady(C_PredictionList* list, void* user_data) {
    auto* engine = static_cast<TypeForgeEngine*>(user_data);

    if (!engine || !engine->instance()) {
        typeforge_free_prediction_list(list);
        return;
    }

    {
        std::lock_guard<std::mutex> lock(engine->queue_mutex_);
        engine->result_queue_.push_back(list);
    }
    
    char c = 1;
    if (write(engine->pipe_fd_[1], &c, 1) < 0) {
        FCITX_WARN() << "Failed to write to pipe";
    }
}

// ---------------------------------------------
// TypeForgeCandidateWord
// ---------------------------------------------

TypeForgeCandidateWord::TypeForgeCandidateWord(std::string text, fcitx::InputContext* ic, TypeForgeEngine* engine)
    : fcitx::CandidateWord(fcitx::Text(text)), text_(std::move(text)), ic_(ic), engine_(engine) {
}

void TypeForgeCandidateWord::select(fcitx::InputContext* ic) const {
    if (engine_) {
        engine_->commitString(ic, text_, true);
    } else {
        ic->inputPanel().reset();
        ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
        ic->commitString(text_);
    }
}
