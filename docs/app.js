// ==============================================================================
// tman — GitHub Pages Interactive Logic
// ==============================================================================

document.addEventListener('DOMContentLoaded', () => {
  // --- 1. Clipboard Copy Buttons ---
  function setupCopyButton(btn, textToCopy) {
    btn.addEventListener('click', async () => {
      try {
        await navigator.clipboard.writeText(textToCopy);
        const originalHtml = btn.innerHTML;
        btn.classList.add('copied');
        
        if (btn.querySelector('.copy-text')) {
          btn.querySelector('.copy-text').textContent = 'Copied!';
        } else {
          btn.textContent = 'Copied!';
        }

        setTimeout(() => {
          btn.classList.remove('copied');
          btn.innerHTML = originalHtml;
        }, 2000);
      } catch (err) {
        console.error('Failed to copy: ', err);
      }
    });
  }

  // Hero Quick Install Copy
  const heroCopyBtn = document.getElementById('hero-copy-btn');
  const installCmd = document.getElementById('install-cmd');
  if (heroCopyBtn && installCmd) {
    setupCopyButton(heroCopyBtn, installCmd.textContent.trim());
  }

  // Code Block Copy Buttons
  document.querySelectorAll('.code-copy-btn').forEach(btn => {
    const text = btn.getAttribute('data-clipboard');
    if (text) {
      setupCopyButton(btn, text);
    }
  });

  // --- 2. Interactive / Screenshot Toggle ---
  const tabInteractive = document.getElementById('tab-interactive');
  const tabScreenshot = document.getElementById('tab-screenshot');
  const interactiveView = document.getElementById('interactive-view');
  const screenshotView = document.getElementById('screenshot-view');

  if (tabInteractive && tabScreenshot) {
    tabInteractive.addEventListener('click', () => {
      tabInteractive.classList.add('active');
      tabScreenshot.classList.remove('active');
      tabInteractive.setAttribute('aria-selected', 'true');
      tabScreenshot.setAttribute('aria-selected', 'false');
      interactiveView.classList.remove('hidden');
      screenshotView.classList.add('hidden');
    });

    tabScreenshot.addEventListener('click', () => {
      tabScreenshot.classList.add('active');
      tabInteractive.classList.remove('active');
      tabScreenshot.setAttribute('aria-selected', 'true');
      tabInteractive.setAttribute('aria-selected', 'false');
      screenshotView.classList.remove('hidden');
      interactiveView.classList.add('hidden');
    });
  }

  // --- 3. Interactive HUD Simulation Logic ---
  const simList = document.getElementById('sim-list');
  const dynTitle = document.getElementById('dyn-block-title');
  const dynContent = document.getElementById('dyn-content');

  const actionStates = {
    navigate: {
      title: 'DIRECTORY CONTENTS',
      items: [
        '▸ src/',
        '▸ tests/',
        '· Cargo.toml',
        '· Dockerfile',
        '· README.md'
      ]
    },
    bookmarks: {
      title: 'PINNED BOOKMARKS',
      items: [
        '* ~/projects/tman',
        '* ~/code/dotfiles',
        '* ~/.config/nvim',
        '* ~/work/backend-api'
      ]
    },
    history: {
      title: 'LIVE SHELL HISTORY (DEDUPLICATED)',
      items: [
        '↺ cargo test --release',
        '↺ git status -sb',
        '↺ docker compose up -d',
        '↺ nvim src/main.rs',
        '↺ git push origin main'
      ]
    },
    search: {
      title: 'FUZZY MATCHES (QUERY: "sys")',
      items: [
        '· src/sys.rs (kernel telemetry)',
        '· tests/test_sys.rs',
        '· src/safety.rs'
      ]
    },
    git: {
      title: 'GIT ACTIONS & STATUS',
      items: [
        '⎇ Status: 2 staged, 1 modified, 0 untracked',
        '⎇ Commit: "feat: add linux support"',
        '⎇ Diff: unstaged changes in src/sys.rs',
        '⎇ Pull (fast-forward preview)',
        '⎇ Push (origin/main)'
      ]
    },
    run: {
      title: 'DISCOVERED WORKFLOW TARGETS',
      items: [
        '▶ cargo build',
        '▶ cargo run -- ui',
        '▶ cargo test',
        '▶ cargo clippy',
        '▶ docker build -t app .'
      ]
    },
    theme: {
      title: 'ACCENT COLOR PALETTE',
      items: [
        '# Cyan (active default)',
        '# Blue (Tokyo Night)',
        '# Green (Matrix)',
        '# Magenta (Synthwave)',
        '# Peach (Catppuccin)'
      ]
    }
  };

  if (simList && dynTitle && dynContent) {
    const items = simList.querySelectorAll('.hud-item');

    function selectAction(item) {
      items.forEach(i => {
        i.classList.remove('active');
        i.querySelector('.item-pointer').textContent = ' ';
      });
      item.classList.add('active');
      item.querySelector('.item-pointer').textContent = '›';

      const actionKey = item.getAttribute('data-action');
      const state = actionStates[actionKey];
      if (state) {
        dynTitle.textContent = state.title;
        dynContent.innerHTML = state.items.map(text => `<div class="dyn-item">${escapeHtml(text)}</div>`).join('');
      }
    }

    items.forEach(item => {
      item.addEventListener('click', () => selectAction(item));
    });

    // Keyboard navigation (Up / Down) when focusing on the simulator
    window.addEventListener('keydown', (e) => {
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
        const activeItem = simList.querySelector('.hud-item.active');
        if (!activeItem) return;
        const all = Array.from(items);
        const idx = all.indexOf(activeItem);
        if (e.key === 'ArrowDown' && idx < all.length - 1) {
          e.preventDefault();
          selectAction(all[idx + 1]);
        } else if (e.key === 'ArrowUp' && idx > 0) {
          e.preventDefault();
          selectAction(all[idx - 1]);
        }
      }
    });
  }

  // --- 4. Live Subtle Telemetry Pulse ---
  const vitalLoad = document.getElementById('vital-load');
  if (vitalLoad) {
    setInterval(() => {
      const l1 = (1.2 + Math.random() * 0.4).toFixed(2);
      const l2 = (1.1 + Math.random() * 0.3).toFixed(2);
      const l3 = (1.0 + Math.random() * 0.2).toFixed(2);
      vitalLoad.textContent = `${l1} ${l2} ${l3}`;
    }, 3500);
  }

  // --- 5. Installation Tabs Switcher ---
  const instTabs = document.querySelectorAll('.inst-tab');
  const tabContents = document.querySelectorAll('.install-tab-content');

  instTabs.forEach(tab => {
    tab.addEventListener('click', () => {
      const targetId = `tab-${tab.getAttribute('data-tab')}`;
      
      instTabs.forEach(t => t.classList.remove('active'));
      tabContents.forEach(c => c.classList.remove('active'));

      tab.classList.add('active');
      const targetContent = document.getElementById(targetId);
      if (targetContent) {
        targetContent.classList.add('active');
      }
    });
  });

  // --- 6. Distro Pill Switcher (inside Linux tab) ---
  const distroPillsContainer = document.getElementById('distro-pills');
  if (distroPillsContainer) {
    distroPillsContainer.addEventListener('click', (e) => {
      const pill = e.target.closest('.distro-pill');
      if (!pill) return;
      const distro = pill.getAttribute('data-distro');

      // Update active pill
      distroPillsContainer.querySelectorAll('.distro-pill').forEach(p => p.classList.remove('active'));
      pill.classList.add('active');

      // Show matching content panel
      document.querySelectorAll('.distro-content').forEach(dc => dc.classList.remove('active'));
      const panel = document.getElementById(`dc-${distro}`);
      if (panel) panel.classList.add('active');
    });
  }

  function escapeHtml(str) {
    return str.replace(/[&<>'"]/g, 
      tag => ({
        '&': '&amp;',
        '<': '&lt;',
        '>': '&gt;',
        "'": '&#39;',
        '"': '&quot;'
      }[tag] || tag)
    );
  }
});
