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
  const rightPanelTitle = document.getElementById('right-panel-title');

  const themeColors = {
    cyan: '#46d7e6',
    blue: '#5aa0ff',
    green: '#50dc96',
    magenta: '#c878ff',
    peach: '#ffb48c'
  };

  const actionStates = {
    navigate: {
      panel: 'MISSION CONTROL',
      title: '━━━ DIRECTORY CONTENTS ━━━━━━━━━━━━━━━',
      items: [
        '▸ src/',
        '▸ tests/',
        '  Cargo.toml',
        '  Dockerfile',
        '  README.md'
      ]
    },
    bookmarks: {
      panel: 'PREVIEW  BOOKMARKS',
      title: '━━━ PINNED & RECENT (ZOXIDE) ━━━━━━━━━',
      items: [
        '* ~/projects/tman',
        '* ~/code/dotfiles',
        '↺ ~/.config/nvim  [recent]',
        '↺ ~/work/backend-api  [recent]'
      ]
    },
    history: {
      panel: 'PREVIEW  HISTORY',
      title: '━━━ LIVE SHELL HISTORY (DEDUPLICATED) ━',
      items: [
        '· cargo test --release',
        '· git status -sb',
        '· docker compose up -d',
        '· nvim src/main.rs',
        '· git push fork linux_ins'
      ]
    },
    search: {
      panel: 'PREVIEW  SEARCH',
      title: '━━━ GITIGNORE-AWARE FUZZY MATCHES ━━━━',
      items: [
        '· src/sys.rs',
        '· src/safety.rs',
        '· src/git.rs',
        '· files/install.sh'
      ]
    },
    git: {
      panel: 'PREVIEW  GIT',
      title: '━━━ INTERACTIVE GIT WORKFLOWS ━━━━━━━━',
      items: [
        '⎇ Changed files  (Space to stage/unstage)',
        '⎇ Branches       (git switch + log preview)',
        '⎇ Stashes        (git stash pop + diff)',
        '+185 -79 in src/app.rs, src/git.rs'
      ]
    },
    run: {
      panel: 'PREVIEW  RUN',
      title: '━━━ DISCOVERED PROJECT TARGETS ━━━━━━━',
      items: [
        '▶ cargo build          Cargo.toml',
        '▶ cargo test           Cargo.toml',
        '▶ docker compose up -d Docker Compose',
        '▶ ${EDITOR:-vi} .env   .env (DATABASE_URL = ***)'
      ]
    },
    ports: {
      panel: 'PREVIEW  ACTIVE PORTS',
      title: '━━━ LISTENING TCP PORTS (LSOF / SS) ━━',
      items: [
        '◈ :3000   node          PID 41230',
        '◈ :5432   postgres      PID 1892',
        '◈ :8080   cargo-watch   PID 50211'
      ]
    },
    theme: {
      panel: 'PREVIEW  THEMES',
      title: '━━━ CLICK A THEME TO PREVIEW ━━━━━━━━━',
      items: [
        '# cyan     Futuristic high-tech cyan [active]',
        '# blue     Electric royal blue',
        '# green    Matrix / terminal emerald green',
        '# magenta  Cyberpunk neon magenta',
        '# peach    Soft warm aesthetic peach'
      ]
    }
  };

  if (simList && dynTitle && dynContent) {
    const items = simList.querySelectorAll('.hud-item');
    let hudHovered = false;

    if (interactiveView) {
      interactiveView.addEventListener('mouseenter', () => { hudHovered = true; });
      interactiveView.addEventListener('mouseleave', () => { hudHovered = false; });
    }

    function selectAction(item) {
      items.forEach(i => {
        i.classList.remove('active');
        i.querySelector('.item-pointer').textContent = ' ';
      });
      item.classList.add('active');
      item.querySelector('.item-pointer').textContent = '❯';

      const actionKey = item.getAttribute('data-action');
      const state = actionStates[actionKey];
      if (state) {
        if (rightPanelTitle) rightPanelTitle.textContent = state.panel;
        dynTitle.textContent = state.title;
        dynContent.innerHTML = state.items.map(text => {
          const themeMatch = actionKey === 'theme' ? text.match(/^#\s+(\w+)/) : null;
          const attr = themeMatch ? ` data-theme-color="${themeMatch[1]}" style="cursor:pointer"` : '';
          return `<div class="dyn-item"${attr}>${escapeHtml(text)}</div>`;
        }).join('');

        if (actionKey === 'theme') {
          dynContent.querySelectorAll('[data-theme-color]').forEach(el => {
            el.addEventListener('click', () => {
              const c = themeColors[el.getAttribute('data-theme-color')];
              if (c) document.documentElement.style.setProperty('--cyan-primary', c);
            });
          });
        }
      }
    }

    items.forEach(item => {
      item.addEventListener('click', () => selectAction(item));
    });

    // Keyboard navigation (Up/Down or j/k) only when hovering or focusing the simulator
    window.addEventListener('keydown', (e) => {
      const focused = document.activeElement === interactiveView || (interactiveView && interactiveView.contains(document.activeElement));
      if (!hudHovered && !focused) return;
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp' || e.key === 'j' || e.key === 'k') {
        const activeItem = simList.querySelector('.hud-item.active');
        if (!activeItem) return;
        const all = Array.from(items);
        const idx = all.indexOf(activeItem);
        if ((e.key === 'ArrowDown' || e.key === 'j') && idx < all.length - 1) {
          e.preventDefault();
          selectAction(all[idx + 1]);
        } else if ((e.key === 'ArrowUp' || e.key === 'k') && idx > 0) {
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
