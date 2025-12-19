const { invoke } = (window as any).__TAURI__.core;

interface PermissionStatus {
  microphone: 'granted' | 'denied' | 'undetermined';
  accessibility: boolean;
}

interface MicrophoneDevice {
  id: string;
  name: string;
  is_default: boolean;
}

// DOM Elements
const micStatus = document.getElementById('mic-status')!;
const accessibilityStatus = document.getElementById('accessibility-status')!;
const btnRequestMic = document.getElementById('btn-request-mic') as HTMLButtonElement;
const btnOpenAccessibility = document.getElementById('btn-open-accessibility') as HTMLButtonElement;
const btnContinue = document.getElementById('btn-continue') as HTMLButtonElement;
const btnFinish = document.getElementById('btn-finish') as HTMLButtonElement;
const micSelector = document.getElementById('mic-selector')!;
const micDropdown = document.getElementById('mic-dropdown') as HTMLSelectElement;

const stepPermissions = document.getElementById('step-permissions')!;
const stepReady = document.getElementById('step-ready')!;
const dot1 = document.getElementById('dot-1')!;
const dot2 = document.getElementById('dot-2')!;

let micPermissionGranted = false;
let accessibilityGranted = false;

// Update status display
function updateMicStatus(status: 'granted' | 'denied' | 'undetermined') {
  const dot = micStatus.querySelector('.status-dot')!;
  const text = micStatus.querySelector('span:last-child')!;

  if (status === 'granted') {
    dot.className = 'status-dot granted';
    text.textContent = 'Granted';
    btnRequestMic.textContent = 'Microphone Access Granted';
    btnRequestMic.disabled = true;
    btnRequestMic.classList.remove('btn-primary');
    btnRequestMic.classList.add('btn-success');
    micPermissionGranted = true;
  } else if (status === 'denied') {
    dot.className = 'status-dot denied';
    text.textContent = 'Denied';
    btnRequestMic.textContent = 'Open System Settings';
  } else {
    dot.className = 'status-dot pending';
    text.textContent = 'Not Granted';
  }

  checkCanContinue();
}

function updateAccessibilityStatus(granted: boolean) {
  const dot = accessibilityStatus.querySelector('.status-dot')!;
  const text = accessibilityStatus.querySelector('span:last-child')!;

  if (granted) {
    dot.className = 'status-dot granted';
    text.textContent = 'Granted';
    btnOpenAccessibility.textContent = 'Accessibility Granted';
    btnOpenAccessibility.disabled = true;
    btnOpenAccessibility.classList.remove('btn-secondary');
    btnOpenAccessibility.classList.add('btn-success');
    accessibilityGranted = true;
  } else {
    dot.className = 'status-dot pending';
    text.textContent = 'Not Granted';
  }

  checkCanContinue();
}

function checkCanContinue() {
  // Can continue if microphone is granted (accessibility is optional but recommended)
  btnContinue.disabled = !micPermissionGranted;

  if (micPermissionGranted && !accessibilityGranted) {
    btnContinue.textContent = 'Continue (auto-paste won\'t work)';
  } else if (micPermissionGranted && accessibilityGranted) {
    btnContinue.textContent = 'Continue';
  }
}

async function loadMicrophones() {
  try {
    const devices: MicrophoneDevice[] = await invoke('get_microphones');
    console.log('Available microphones:', devices);

    micDropdown.innerHTML = '';

    for (const device of devices) {
      const option = document.createElement('option');
      option.value = device.id;
      option.textContent = device.name;
      if (device.is_default) {
        option.selected = true;
      }
      micDropdown.appendChild(option);
    }

    micSelector.classList.remove('hidden');
  } catch (error) {
    console.error('Failed to load microphones:', error);
  }
}

async function checkPermissions() {
  try {
    const status: PermissionStatus = await invoke('check_permissions');
    console.log('Permission status:', status);

    updateMicStatus(status.microphone);
    updateAccessibilityStatus(status.accessibility);

    if (status.microphone === 'granted') {
      loadMicrophones();
    }
  } catch (error) {
    console.error('Failed to check permissions:', error);
  }
}

// Request microphone permission
btnRequestMic.addEventListener('click', async () => {
  try {
    const result = await invoke('request_microphone_permission');
    console.log('Microphone permission result:', result);

    // Re-check permissions after request
    setTimeout(checkPermissions, 500);
  } catch (error) {
    console.error('Failed to request microphone permission:', error);
  }
});

// Open accessibility settings
btnOpenAccessibility.addEventListener('click', async () => {
  try {
    await invoke('open_accessibility_settings');

    // Start polling for accessibility permission
    const pollInterval = setInterval(async () => {
      try {
        const status: PermissionStatus = await invoke('check_permissions');
        if (status.accessibility) {
          clearInterval(pollInterval);
          updateAccessibilityStatus(true);
        }
      } catch (e) {
        console.error('Poll error:', e);
      }
    }, 1000);

    // Stop polling after 60 seconds
    setTimeout(() => clearInterval(pollInterval), 60000);
  } catch (error) {
    console.error('Failed to open accessibility settings:', error);
  }
});

// Save microphone selection
micDropdown.addEventListener('change', async () => {
  try {
    await invoke('set_selected_microphone', { deviceId: micDropdown.value });
    console.log('Selected microphone:', micDropdown.value);
  } catch (error) {
    console.error('Failed to set microphone:', error);
  }
});

// Continue to next step
btnContinue.addEventListener('click', () => {
  stepPermissions.classList.remove('active');
  stepReady.classList.add('active');
  dot1.classList.remove('active');
  dot1.classList.add('completed');
  dot2.classList.add('active');
});

// Finish onboarding
btnFinish.addEventListener('click', async () => {
  try {
    await invoke('complete_onboarding');
  } catch (error) {
    console.error('Failed to complete onboarding:', error);
  }
});

// Initial permission check
checkPermissions();

// Poll for permission changes (in case user grants via System Settings)
setInterval(checkPermissions, 3000);

console.log('Onboarding initialized');
