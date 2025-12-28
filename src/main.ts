import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface Preferences {
  recordingMode: string;
  hotkey: string;
  showIndicator: boolean;
  playSounds: boolean;
  microphone: string;
  language: string;
  deepgramKey: string;
  claudeKey: string;
  transcriptionProvider?: string;
  licenseKey?: string;
  onboardingComplete?: boolean;
  spokenLanguages?: string[];
}

interface ModelStatus {
  downloaded: boolean;
  downloading: boolean;
  progress: number;
  size_bytes: number;
  path: string | null;
}

interface LicenseInfo {
  tier: "free" | "subscription" | "lifetime";
  valid: boolean;
  license_key: string | null;
  expires_at: string | null;
}

// Recording state - synced from backend
type RecordingState = "idle" | "recording" | "transcribing" | "enhancing" | "error";
let currentState: RecordingState = "idle";

// Initialize preferences
let preferences: Preferences = {
  recordingMode: "toggle",
  hotkey: "Cmd+Shift+D",
  showIndicator: true,
  playSounds: true,
  microphone: "default",
  language: "en-US",
  deepgramKey: "",
  claudeKey: "",
  transcriptionProvider: "deepgram",
  licenseKey: "",
  onboardingComplete: false,
  spokenLanguages: ["en"]
};

// Current provider state
let currentProvider = "deepgram";
let modelStatus: ModelStatus = {
  downloaded: false,
  downloading: false,
  progress: 0,
  size_bytes: 0,
  path: null
};
let licenseInfo: LicenseInfo = {
  tier: "free",
  valid: false,
  license_key: null,
  expires_at: null
};

// Load preferences from backend (which reads from persistent file)
async function loadPreferences() {
  try {
    // Load from backend - this reads from ~/.config/murmur/preferences.json
    const stored = await invoke<Preferences>("get_preferences");
    preferences = { ...preferences, ...stored };
    console.log("Loaded preferences from backend:", preferences);
    applyPreferences();
    checkOnboarding();
  } catch (error) {
    console.error("Error loading preferences from backend:", error);
    // Fallback to localStorage for migration
    try {
      const localStored = localStorage.getItem("preferences");
      if (localStored) {
        preferences = { ...preferences, ...JSON.parse(localStored) };
        applyPreferences();
        console.log("Loaded preferences from localStorage (fallback)");
      }
    } catch (localError) {
      console.error("Error loading from localStorage:", localError);
    }
    // Show onboarding for first-time users
    checkOnboarding();
  }
}

// Check and show onboarding if needed
function checkOnboarding() {
  const onboardingOverlay = document.getElementById('onboarding-overlay');
  if (!onboardingOverlay) return;

  if (!preferences.onboardingComplete) {
    onboardingOverlay.classList.remove('hidden');
  } else {
    onboardingOverlay.classList.add('hidden');
  }
}

// Complete language onboarding and save selected languages
async function completeOnboarding() {
  const checkboxes = document.querySelectorAll('input[name="onboard-lang"]:checked') as NodeListOf<HTMLInputElement>;
  const selectedLanguages = Array.from(checkboxes).map(cb => cb.value);

  // Default to English if nothing selected
  if (selectedLanguages.length === 0) {
    selectedLanguages.push('en');
  }

  preferences.spokenLanguages = selectedLanguages;
  preferences.onboardingComplete = true;

  // Keep language as English (default) - user can switch to Mixed mode if they want multilingual
  // preferences.language stays at "en-US" by default

  // Save preferences (this stores language onboarding completion in preferences.json)
  try {
    await invoke("update_preferences", { preferences });
    console.log("Language onboarding complete, preferences saved with spoken languages:", selectedLanguages);
  } catch (error) {
    console.error("Error saving preferences:", error);
  }

  // Hide onboarding overlay
  const onboardingOverlay = document.getElementById('onboarding-overlay');
  if (onboardingOverlay) {
    onboardingOverlay.classList.add('hidden');
  }

  // Apply preferences to UI
  applyPreferences();
}

// Setup onboarding event listeners
function setupOnboarding() {
  // Language checkbox toggle styling
  const languageCheckboxes = document.querySelectorAll('.language-checkbox');
  languageCheckboxes.forEach(label => {
    const checkbox = label.querySelector('input[type="checkbox"]') as HTMLInputElement;
    if (checkbox) {
      // Set initial state
      if (checkbox.checked) {
        label.classList.add('selected');
      }

      checkbox.addEventListener('change', () => {
        if (checkbox.checked) {
          label.classList.add('selected');
        } else {
          label.classList.remove('selected');
        }
      });
    }
  });

  // Get Started button
  const continueBtn = document.getElementById('onboarding-continue');
  if (continueBtn) {
    continueBtn.addEventListener('click', completeOnboarding);
  }
}

// Apply preferences to UI
function applyPreferences() {
  const recordingMode = document.getElementById("recording-mode") as HTMLSelectElement;
  const hotkey = document.getElementById("hotkey") as HTMLSelectElement;
  const showIndicator = document.getElementById("show-indicator") as HTMLInputElement;
  const playSounds = document.getElementById("play-sounds") as HTMLInputElement;
  const language = document.getElementById("language") as HTMLSelectElement;
  const deepgramKey = document.getElementById("deepgram-key") as HTMLInputElement;
  const claudeKey = document.getElementById("claude-key") as HTMLInputElement;

  if (recordingMode) recordingMode.value = preferences.recordingMode;
  if (hotkey) hotkey.value = preferences.hotkey;
  if (showIndicator) showIndicator.checked = preferences.showIndicator;
  if (playSounds) playSounds.checked = preferences.playSounds;
  if (language) language.value = preferences.language;
  if (deepgramKey) deepgramKey.value = preferences.deepgramKey;
  if (claudeKey) claudeKey.value = preferences.claudeKey;

  // Apply transcription provider
  if (preferences.transcriptionProvider) {
    currentProvider = preferences.transcriptionProvider;
    selectProvider(currentProvider);
  }

  // Apply license key
  const licenseKeyInput = document.getElementById("license-key") as HTMLInputElement;
  if (licenseKeyInput && preferences.licenseKey) {
    licenseKeyInput.value = preferences.licenseKey;
  }

  // Update spoken languages display
  updateSpokenLanguagesDisplay();
}

// Language code to display name mapping
const languageNames: Record<string, string> = {
  en: "English",
  hi: "Hindi",
  te: "Telugu",
  ta: "Tamil",
  es: "Spanish",
  fr: "French",
  de: "German",
  ja: "Japanese",
  zh: "Chinese",
  ko: "Korean",
};

// Update the spoken languages chips display
function updateSpokenLanguagesDisplay() {
  const display = document.getElementById("spoken-languages-display");
  if (!display) return;

  // Clear existing content safely
  display.textContent = '';

  const languages = preferences.spokenLanguages || ["en"];
  // Use DOM manipulation instead of innerHTML to prevent XSS
  languages.forEach(lang => {
    const chip = document.createElement('span');
    chip.className = 'language-chip';
    // textContent automatically escapes HTML characters
    chip.textContent = languageNames[lang] || lang;
    display.appendChild(chip);
  });
}

// Setup spoken languages editor
function setupSpokenLanguagesEditor() {
  const editBtn = document.getElementById("edit-languages-btn");
  const saveBtn = document.getElementById("save-languages-btn");
  const editor = document.getElementById("spoken-languages-editor");

  if (editBtn && editor) {
    editBtn.addEventListener("click", () => {
      editor.classList.toggle("hidden");
      if (!editor.classList.contains("hidden")) {
        // Sync checkboxes with current preferences
        const checkboxes = document.querySelectorAll('input[name="spoken-lang"]') as NodeListOf<HTMLInputElement>;
        const currentLangs = preferences.spokenLanguages || ["en"];
        checkboxes.forEach(cb => {
          cb.checked = currentLangs.includes(cb.value);
          const label = cb.closest(".language-checkbox");
          if (label) {
            if (cb.checked) {
              label.classList.add("selected");
            } else {
              label.classList.remove("selected");
            }
          }
        });
      }
    });
  }

  if (saveBtn && editor) {
    saveBtn.addEventListener("click", async () => {
      const checkboxes = document.querySelectorAll('input[name="spoken-lang"]:checked') as NodeListOf<HTMLInputElement>;
      const selectedLanguages = Array.from(checkboxes).map(cb => cb.value);

      // Default to English if nothing selected
      if (selectedLanguages.length === 0) {
        selectedLanguages.push("en");
      }

      preferences.spokenLanguages = selectedLanguages;
      updateSpokenLanguagesDisplay();
      editor.classList.add("hidden");

      // Save to backend
      try {
        await invoke("update_preferences", { preferences });
        console.log("Spoken languages updated:", selectedLanguages);
      } catch (error) {
        console.error("Error saving spoken languages:", error);
      }
    });
  }

  // Setup checkbox toggle styling for the editor
  const editorCheckboxes = document.querySelectorAll('#spoken-languages-editor .language-checkbox');
  editorCheckboxes.forEach(label => {
    const checkbox = label.querySelector('input[type="checkbox"]') as HTMLInputElement;
    if (checkbox) {
      checkbox.addEventListener('change', () => {
        if (checkbox.checked) {
          label.classList.add('selected');
        } else {
          label.classList.remove('selected');
        }
      });
    }
  });
}

// Select a transcription provider
function selectProvider(provider: string) {
  currentProvider = provider;

  // Update radio buttons
  const radios = document.querySelectorAll('input[name="provider"]') as NodeListOf<HTMLInputElement>;
  radios.forEach(radio => {
    radio.checked = radio.value === provider;
  });

  // Update card selection styles
  const cards = document.querySelectorAll('.provider-card');
  cards.forEach(card => {
    const cardProvider = card.getAttribute('data-provider');
    if (cardProvider === provider) {
      card.classList.add('selected');
    } else {
      card.classList.remove('selected');
    }
  });

  // Show/hide license section
  const licenseSection = document.getElementById('license-section');
  if (licenseSection) {
    if (provider === 'whisperapi' || provider === 'whisperlocal') {
      licenseSection.classList.remove('hidden');
    } else {
      licenseSection.classList.add('hidden');
    }
  }

  // Show/hide model section
  const modelSection = document.getElementById('model-section');
  if (modelSection) {
    if (provider === 'whisperlocal') {
      modelSection.classList.remove('hidden');
      loadModelStatus();
    } else {
      modelSection.classList.add('hidden');
    }
  }

  // Save provider to backend
  invoke('set_transcription_provider', { provider }).catch(console.error);
}

// Load model status from backend
async function loadModelStatus() {
  try {
    modelStatus = await invoke<ModelStatus>('get_model_status');
    updateModelUI();
  } catch (error) {
    console.error('Failed to load model status:', error);
  }
}

// Update model UI based on status
function updateModelUI() {
  const statusText = document.getElementById('model-status-text');
  const progressContainer = document.getElementById('model-progress');
  const progressFill = document.getElementById('progress-fill');
  const progressText = document.getElementById('progress-text');
  const downloadBtn = document.getElementById('download-model') as HTMLButtonElement;
  const deleteBtn = document.getElementById('delete-model') as HTMLButtonElement;
  const modelSize = document.getElementById('model-size');

  if (!statusText || !progressContainer || !downloadBtn || !deleteBtn) return;

  if (modelStatus.downloading) {
    statusText.textContent = 'Downloading...';
    statusText.className = 'model-status downloading';
    progressContainer.classList.remove('hidden');
    if (progressFill) progressFill.style.width = `${modelStatus.progress}%`;
    if (progressText) progressText.textContent = `${Math.round(modelStatus.progress)}%`;
    downloadBtn.disabled = true;
    downloadBtn.textContent = 'Downloading...';
    deleteBtn.classList.add('hidden');
  } else if (modelStatus.downloaded) {
    statusText.textContent = 'Downloaded and ready';
    statusText.className = 'model-status downloaded';
    progressContainer.classList.add('hidden');
    downloadBtn.classList.add('hidden');
    deleteBtn.classList.remove('hidden');
  } else {
    statusText.textContent = 'Not downloaded';
    statusText.className = 'model-status';
    progressContainer.classList.add('hidden');
    downloadBtn.classList.remove('hidden');
    downloadBtn.disabled = false;
    downloadBtn.textContent = 'Download Model';
    deleteBtn.classList.add('hidden');
  }

  // Update size display
  if (modelSize && modelStatus.size_bytes > 0) {
    const sizeMB = Math.round(modelStatus.size_bytes / (1024 * 1024));
    modelSize.textContent = `${sizeMB} MB`;
  }
}

// Download model
async function downloadModel() {
  const downloadBtn = document.getElementById('download-model') as HTMLButtonElement;
  if (downloadBtn) {
    downloadBtn.disabled = true;
    downloadBtn.textContent = 'Starting download...';
  }

  try {
    await invoke('download_model');
    await loadModelStatus();
  } catch (error) {
    console.error('Failed to download model:', error);
    alert(`Failed to download model: ${error}`);
    if (downloadBtn) {
      downloadBtn.disabled = false;
      downloadBtn.textContent = 'Download Model';
    }
  }
}

// Delete model
async function deleteModel() {
  if (!confirm('Are you sure you want to delete the Whisper model? You will need to download it again to use local transcription.')) {
    return;
  }

  try {
    await invoke('delete_model');
    await loadModelStatus();
  } catch (error) {
    console.error('Failed to delete model:', error);
    alert(`Failed to delete model: ${error}`);
  }
}

// Activate license
async function activateLicense() {
  const licenseKeyInput = document.getElementById('license-key') as HTMLInputElement;
  const licenseStatus = document.getElementById('license-status');
  const activateBtn = document.getElementById('activate-license') as HTMLButtonElement;

  if (!licenseKeyInput || !licenseStatus) return;

  const key = licenseKeyInput.value.trim();
  if (!key) {
    licenseStatus.textContent = 'Please enter a license key';
    licenseStatus.className = 'license-status invalid';
    return;
  }

  licenseStatus.textContent = 'Validating...';
  licenseStatus.className = 'license-status checking';
  if (activateBtn) activateBtn.disabled = true;

  try {
    licenseInfo = await invoke<LicenseInfo>('activate_license', { licenseKey: key });

    if (licenseInfo.valid) {
      const tierText = licenseInfo.tier === 'subscription' ? 'Subscription' : 'Lifetime';
      licenseStatus.textContent = `License activated: ${tierText}`;
      licenseStatus.className = 'license-status valid';
      preferences.licenseKey = key;

      // Auto-select the appropriate provider based on license
      if (licenseInfo.tier === 'subscription') {
        selectProvider('whisperapi');
      } else if (licenseInfo.tier === 'lifetime') {
        selectProvider('whisperlocal');
      }
    } else {
      licenseStatus.textContent = 'Invalid license key';
      licenseStatus.className = 'license-status invalid';
    }
  } catch (error) {
    console.error('Failed to activate license:', error);
    licenseStatus.textContent = `Activation failed: ${error}`;
    licenseStatus.className = 'license-status invalid';
  } finally {
    if (activateBtn) activateBtn.disabled = false;
  }
}

// Load license info from cache
async function loadLicenseInfo() {
  try {
    licenseInfo = await invoke<LicenseInfo>('get_license_info');
    updateLicenseUI();
  } catch (error) {
    console.error('Failed to load license info:', error);
  }
}

// Update license UI
function updateLicenseUI() {
  const licenseStatus = document.getElementById('license-status');
  if (!licenseStatus) return;

  if (licenseInfo.valid) {
    const tierText = licenseInfo.tier === 'subscription' ? 'Subscription' : 'Lifetime';
    licenseStatus.textContent = `License active: ${tierText}`;
    licenseStatus.className = 'license-status valid';
  }
}

// Tab switching functionality
function setupTabs() {
  const tabButtons = document.querySelectorAll(".tab-button");
  const tabContents = document.querySelectorAll(".tab-content");

  tabButtons.forEach(button => {
    button.addEventListener("click", () => {
      const targetTab = button.getAttribute("data-tab");

      // Update active states
      tabButtons.forEach(btn => btn.classList.remove("active"));
      tabContents.forEach(content => content.classList.remove("active"));

      button.classList.add("active");
      const targetContent = document.getElementById(`${targetTab}-tab`);
      if (targetContent) {
        targetContent.classList.add("active");
      }
    });
  });
}

// Update test recording button based on state
function updateTestButton() {
  const testRecordingButton = document.getElementById("test-recording");
  if (!testRecordingButton) return;

  switch (currentState) {
    case "idle":
    case "error":
      testRecordingButton.textContent = "Test Recording";
      testRecordingButton.removeAttribute("disabled");
      break;
    case "recording":
      testRecordingButton.textContent = "Stop Recording";
      testRecordingButton.removeAttribute("disabled");
      break;
    case "transcribing":
    case "enhancing":
      testRecordingButton.textContent = "Processing...";
      testRecordingButton.setAttribute("disabled", "true");
      break;
  }
}

// Handle recording toggle - just call backend, let it manage state
async function handleRecordingToggle() {
  try {
    await invoke("toggle_recording");
  } catch (error) {
    console.error("Error toggling recording:", error);
  }
}

// Setup button event listeners
function setupEventListeners() {
  // Save button
  const saveButton = document.getElementById("save-preferences");
  if (saveButton) {
    saveButton.addEventListener("click", async () => {
      const recordingMode = document.getElementById("recording-mode") as HTMLSelectElement;
      const hotkey = document.getElementById("hotkey") as HTMLSelectElement;
      const showIndicator = document.getElementById("show-indicator") as HTMLInputElement;
      const playSounds = document.getElementById("play-sounds") as HTMLInputElement;
      const language = document.getElementById("language") as HTMLSelectElement;
      const deepgramKey = document.getElementById("deepgram-key") as HTMLInputElement;
      const claudeKey = document.getElementById("claude-key") as HTMLInputElement;

      const licenseKeyInput = document.getElementById("license-key") as HTMLInputElement;

      preferences = {
        recordingMode: recordingMode.value,
        hotkey: hotkey.value,
        showIndicator: showIndicator.checked,
        playSounds: playSounds.checked,
        microphone: preferences.microphone,
        language: language.value,
        deepgramKey: deepgramKey.value,
        claudeKey: claudeKey.value,
        transcriptionProvider: currentProvider,
        licenseKey: licenseKeyInput?.value || preferences.licenseKey,
        // Preserve onboarding settings
        onboardingComplete: preferences.onboardingComplete,
        spokenLanguages: preferences.spokenLanguages
      };

      // Save through backend (persists to ~/.config/murmur/preferences.json)
      try {
        // Show saving state on button
        saveButton.textContent = "Saving...";
        saveButton.setAttribute("disabled", "true");

        await invoke("update_preferences", { preferences });
        console.log("Preferences saved successfully");

        // Show success feedback briefly
        saveButton.textContent = "✓ Saved!";
        saveButton.classList.add("success");

        // Wait a moment to show feedback, then close
        await new Promise(resolve => setTimeout(resolve, 500));

        const window = getCurrentWindow();
        await window.hide();

        // Reset button state for next time
        saveButton.textContent = "Save";
        saveButton.removeAttribute("disabled");
        saveButton.classList.remove("success");
      } catch (error) {
        console.error("Error saving preferences:", error);
        saveButton.textContent = "Save";
        saveButton.removeAttribute("disabled");
        alert(`Failed to save preferences: ${error}`);
        return;
      }
    });
  }

  // Cancel button
  const cancelButton = document.getElementById("cancel-preferences");
  if (cancelButton) {
    cancelButton.addEventListener("click", async () => {
      applyPreferences();
      const window = getCurrentWindow();
      await window.hide();
    });
  }

  // Test API button
  const testApisButton = document.getElementById("test-apis");
  if (testApisButton) {
    testApisButton.addEventListener("click", async () => {
      const statusDiv = document.getElementById("api-status");
      const deepgramKeyInput = document.getElementById("deepgram-key") as HTMLInputElement;
      const claudeKeyInput = document.getElementById("claude-key") as HTMLInputElement;
      if (!statusDiv) return;

      const deepgramKey = deepgramKeyInput?.value || "";
      const claudeKey = claudeKeyInput?.value || "";

      const results: string[] = [];
      let hasErrors = false;

      statusDiv.textContent = "Testing API connections...";
      statusDiv.className = "status-message";

      // Test Deepgram API
      if (deepgramKey) {
        try {
          const response = await fetch("https://api.deepgram.com/v1/projects", {
            method: "GET",
            headers: {
              "Authorization": `Token ${deepgramKey}`
            }
          });
          if (response.ok) {
            results.push("✓ Deepgram API key valid");
          } else {
            results.push("✗ Deepgram API key invalid");
            hasErrors = true;
          }
        } catch {
          results.push("✗ Deepgram connection failed");
          hasErrors = true;
        }
      } else {
        results.push("✗ Deepgram API key not entered");
        hasErrors = true;
      }

      // Test Claude API
      if (claudeKey) {
        try {
          const response = await fetch("https://api.anthropic.com/v1/messages", {
            method: "POST",
            headers: {
              "x-api-key": claudeKey,
              "anthropic-version": "2023-06-01",
              "content-type": "application/json",
              "anthropic-dangerous-direct-browser-access": "true"
            },
            body: JSON.stringify({
              model: "claude-3-5-sonnet-20241022",
              max_tokens: 10,
              messages: [{ role: "user", content: "Hi" }]
            })
          });
          if (response.ok) {
            results.push("✓ Claude API key valid");
          } else {
            const data = await response.json();
            if (data.error?.message?.includes("credit")) {
              results.push("⚠ Claude API key valid but no credits");
            } else {
              results.push("✗ Claude API key invalid");
              hasErrors = true;
            }
          }
        } catch {
          results.push("✗ Claude connection failed");
          hasErrors = true;
        }
      } else {
        results.push("✗ Claude API key not entered");
        hasErrors = true;
      }

      statusDiv.innerHTML = results.join("<br>");
      statusDiv.className = hasErrors ? "status-message error" : "status-message success";
    });
  }

  // Test hotkey button
  const testHotkeyButton = document.getElementById("test-hotkey");
  if (testHotkeyButton) {
    testHotkeyButton.addEventListener("click", () => {
      alert(`Press ${preferences.hotkey} to start recording`);
    });
  }

  // Test recording button
  const testRecordingButton = document.getElementById("test-recording");
  if (testRecordingButton) {
    testRecordingButton.addEventListener("click", handleRecordingToggle);
  }

  // Provider card click handlers
  const providerCards = document.querySelectorAll('.provider-card');
  providerCards.forEach(card => {
    card.addEventListener('click', () => {
      const provider = card.getAttribute('data-provider');
      if (provider) {
        selectProvider(provider);
      }
    });
  });

  // Provider radio button change handlers
  const providerRadios = document.querySelectorAll('input[name="provider"]') as NodeListOf<HTMLInputElement>;
  providerRadios.forEach(radio => {
    radio.addEventListener('change', () => {
      if (radio.checked) {
        selectProvider(radio.value);
      }
    });
  });

  // License activation button
  const activateLicenseBtn = document.getElementById('activate-license');
  if (activateLicenseBtn) {
    activateLicenseBtn.addEventListener('click', activateLicense);
  }

  // License key input - activate on Enter
  const licenseKeyInput = document.getElementById('license-key') as HTMLInputElement;
  if (licenseKeyInput) {
    licenseKeyInput.addEventListener('keypress', (e) => {
      if (e.key === 'Enter') {
        activateLicense();
      }
    });
  }

  // Model download button
  const downloadModelBtn = document.getElementById('download-model');
  if (downloadModelBtn) {
    downloadModelBtn.addEventListener('click', downloadModel);
  }

  // Model delete button
  const deleteModelBtn = document.getElementById('delete-model');
  if (deleteModelBtn) {
    deleteModelBtn.addEventListener('click', deleteModel);
  }
}

// Initialize the app
window.addEventListener("DOMContentLoaded", () => {
  setupOnboarding();
  setupSpokenLanguagesEditor();
  loadPreferences();
  setupTabs();
  setupEventListeners();
  loadLicenseInfo();

  // Listen for recording toggle event from backend (menu bar)
  listen("toggle-recording", () => {
    handleRecordingToggle();
  });

  // Listen for model download progress
  listen<{ progress: number; downloaded_bytes: number; total_bytes: number }>("model-download-progress", (event) => {
    modelStatus.downloading = true;
    modelStatus.progress = event.payload.progress;
    modelStatus.size_bytes = event.payload.total_bytes;
    updateModelUI();
  });

  // Listen for model download complete
  listen<{ path: string }>("model-download-complete", (event) => {
    modelStatus.downloading = false;
    modelStatus.downloaded = true;
    modelStatus.path = event.payload.path;
    modelStatus.progress = 100;
    updateModelUI();
  });

  // Listen for model download error
  listen<{ error: string }>("model-download-error", (event) => {
    modelStatus.downloading = false;
    modelStatus.progress = 0;
    updateModelUI();
    alert(`Model download failed: ${event.payload.error}`);
  });

  // Listen for state changes from backend
  listen<{ state: RecordingState; message?: string }>("state-changed", (event) => {
    currentState = event.payload.state;
    console.log("State changed:", currentState, event.payload.message);
    updateTestButton();
  });

  // Listen for transcription complete
  listen<{ enhancedText: string; copiedToClipboard: boolean }>("transcription-complete", (event) => {
    console.log("Transcription complete:", event.payload);
  });

  // Listen for errors
  listen<{ code: string; message: string }>("recording-error", (event) => {
    console.error("Recording error:", event.payload);
    alert(event.payload.message);
  });

  // Listen for global shortcut events
  listen("shortcut-start", async () => {
    console.log("Shortcut start received");
    try {
      await invoke("start_recording");
    } catch (error) {
      console.error("Error starting recording:", error);
    }
  });

  listen("shortcut-stop", async () => {
    console.log("Shortcut stop received");
    try {
      await invoke("stop_recording");
    } catch (error) {
      console.error("Error stopping recording:", error);
    }
  });

  listen("shortcut-toggle", async () => {
    console.log("Shortcut toggle received");
    handleRecordingToggle();
  });
});
