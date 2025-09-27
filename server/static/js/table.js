function renderPage(cards, filteredCards, currentPage, perPage, pageInfo) {
    const start = (currentPage - 1) * perPage;
    const end = start + perPage;
    const cardsToShow = filteredCards.slice(start, end);

    cards.forEach(card => card.style.display = 'none');

    const table = cards[0]?.parentNode;
    if (!table) return;

    cardsToShow.forEach((card, idx) => {
        const orig = card.__origDisplay || 'grid';
        card.style.display = orig;
        table.appendChild(card);

        card.classList.remove('last-row');

        if (idx === cardsToShow.length - 1) {
            card.classList.add('last-row');
        }
    });

    const totalPages = Math.ceil(filteredCards.length / perPage) || 1;
    pageInfo.innerText = `Page ${currentPage} of ${totalPages} (${filteredCards.length} items)`;
}

function tryParseDate(val) {
    if (!val) return null;
    const v = String(val).trim();
    if (!/^\d{4}-\d{2}-\d{2}/.test(v)) return null;

    const iso = v.includes('T') ? v : v.replace(' ', 'T');
    const d = new Date(iso);
    if (!isNaN(d.getTime())) return d;

    const d2 = new Date(v);
    return isNaN(d2.getTime()) ? null : d2;
}

function parseValue(raw) {
    const val = String(raw ?? '').trim();
    if (val === '') return '';

    let numericCandidate = val.replace(/[^\d\-,.]/g, '');

    if (numericCandidate === '') {
        const d = tryParseDate(val);
        if (d) return d;
        return val.toLowerCase();
    }

    if (numericCandidate.indexOf('.') !== -1 && numericCandidate.indexOf(',') !== -1) {
        if (numericCandidate.indexOf('.') < numericCandidate.indexOf(',')) {
            numericCandidate = numericCandidate.replace(/\./g, '').replace(/,/g, '.');
        } else {
            numericCandidate = numericCandidate.replace(/,/g, '');
        }
    } else if (numericCandidate.indexOf(',') !== -1) {
        numericCandidate = numericCandidate.replace(/,/g, '.');
    }

    numericCandidate = numericCandidate.replace(/(?!^-)-/g, '');

    const n = parseFloat(numericCandidate);
    if (!isNaN(n)) return n;

    const d = tryParseDate(val);
    if (d) return d;

    return val.toLowerCase();
}

function sortCards(filteredCards, field, asc, fieldToIndex) {
    const columnIndex = fieldToIndex[field];

    if (columnIndex === undefined) {
        console.error('Field not found in mapping:', field);
        return;
    }

    filteredCards.sort((a, b) => {
        const aEl = a.children[columnIndex];
        const bEl = b.children[columnIndex];

        if (!aEl || !bEl) return 0;

        const aVal = parseValue(aEl.innerText);
        const bVal = parseValue(bEl.innerText);

        let result = 0;
        if (aVal < bVal) result = -1;
        else if (aVal > bVal) result = 1;

        return asc ? result : -result;
    });
}

export class CardsList {
    constructor({
        tableSelector,
        headerSelector,
        searchInputId,
        pageInfoId,
        perPage = 5
    }) {
        this.table = document.querySelector(tableSelector);
        this.header = document.querySelector(headerSelector);
        this.searchInput = document.getElementById(searchInputId);
        this.pageInfo = document.getElementById(pageInfoId);
        this.perPage = perPage;

        if (!this.table || !this.header || !this.searchInput || !this.pageInfo) {
            console.error('One or more required elements not found:', {
                table: !!this.table,
                header: !!this.header,
                searchInput: !!this.searchInput,
                pageInfo: !!this.pageInfo
            });
            return;
        }

        this.cards = Array.from(this.table.querySelectorAll('.table-row'));
        this.cards.forEach(card => {
            if (!card.__origDisplay) {
                const computed = window.getComputedStyle(card).display;
                card.__origDisplay = computed && computed !== 'none' ? computed : 'grid';
            }
        });

        this.filteredCards = [...this.cards];
        this.currentPage = 1;
        this.headers = Array.from(this.header.querySelectorAll('.sortable'));

        this.fieldToIndex = {};
        this.headers.forEach((h, idx) => {
            const field = h.dataset.field;
            if (field) this.fieldToIndex[field] = idx;
        });

        try { window.cardsList = this; } catch (e) { /* ignore if sandboxed */ }

        this.init();
    }

    init() {
        this.setupEventListeners();
        this.renderPage();

        if (this.searchInput.value.trim()) {
            this.applyFilter();
        }
    }

    setupEventListeners() {
        this.searchInput.addEventListener('input', () => {
            this.applyFilter();
        });

        this.headers.forEach(h => {
            h.dataset.asc = h.dataset.asc ?? 'true';
            h.style.cursor = 'pointer';

            h.addEventListener('click', (e) => {
                e.preventDefault();
                this.handleHeaderClick(h);
            });
        });
    }

    renderPage() {
        renderPage(this.cards, this.filteredCards, this.currentPage, this.perPage, this.pageInfo);
    }

    applyFilter() {
        const query = this.searchInput.value.toLowerCase();
        this.filteredCards = this.cards.filter(card => card.innerText.toLowerCase().includes(query));
        this.currentPage = 1;
        this.renderPage();
    }

    handleHeaderClick(header) {
        const field = header.dataset.field;
        const asc = header.dataset.asc === 'true';

        this.headers.forEach(h => {
            const indicator = h.querySelector('.sort-indicator');
            if (indicator) indicator.textContent = '↕';
        });

        const indicator = header.querySelector('.sort-indicator');
        if (indicator) indicator.textContent = asc ? '↓' : '↑';

        header.dataset.asc = (!asc).toString();

        sortCards(this.filteredCards, field, asc, this.fieldToIndex);

        this.currentPage = 1;
        this.renderPage();
    }

    nextPage() {
        const maxPage = Math.ceil(this.filteredCards.length / this.perPage);
        if (this.currentPage < maxPage) {
            this.currentPage++;
            this.renderPage();
        }
    }

    prevPage() {
        if (this.currentPage > 1) {
            this.currentPage--;
            this.renderPage();
        }
    }

    deleteCard(cardElement) {
        const cardIndex = this.cards.indexOf(cardElement);
        if (cardIndex > -1) {
            this.cards.splice(cardIndex, 1);
            cardElement.remove();

            const filteredIndex = this.filteredCards.indexOf(cardElement);
            if (filteredIndex > -1) {
                this.filteredCards.splice(filteredIndex, 1);
            }

            if (this.filteredCards.length === 0 && this.currentPage > 1) {
                this.currentPage = Math.max(1, this.currentPage - 1);
            }

            this.renderPage();
        }
    }

    deleteCardById(cardId) {
        const cardElement = this.cards.find(card => card.dataset.id === cardId);
        if (cardElement) this.deleteCard(cardElement);
    }

    addCard(cardElement) {
        if (!cardElement.__origDisplay) {
            const computed = window.getComputedStyle(cardElement).display;
            cardElement.__origDisplay = computed && computed !== 'none' ? computed : 'grid';
        }
        this.table.appendChild(cardElement);
        this.cards.push(cardElement);
        this.attachListenersToCard(cardElement);
        this.applyFilter();
    }

    refreshCards() {
        this.cards = Array.from(this.table.querySelectorAll('.table-row'));
        this.cards.forEach(card => {
            if (!card.__origDisplay) {
                const computed = window.getComputedStyle(card).display;
                card.__origDisplay = computed && computed !== 'none' ? computed : 'grid';
            }
        });
        this.attachRowButtons(true);
        this.applyFilter();
    }

    getVisibleCards() {
        return this.filteredCards.slice(
            (this.currentPage - 1) * this.perPage,
            this.currentPage * this.perPage
        );
    }

    getTotalPages() {
        return Math.ceil(this.filteredCards.length / this.perPage) || 1;
    }

    getCurrentPage() {
        return this.currentPage;
    }

    setCurrentPage(page) {
        const maxPage = this.getTotalPages();
        if (page >= 1 && page <= maxPage) {
            this.currentPage = page;
            this.renderPage();
        }
    }

    attachRowButtons(force = false) {
        this.cards.forEach(card => {
            this.attachListenersToCard(card, force);
        });
    }

    attachListenersToCard(card, force = false) {
        if (!force && card.__listenersAttached) return;

        const deleteBtn = card.querySelector('.delete-btn');
        if (deleteBtn && !deleteBtn.getAttribute('onclick')) {
            deleteBtn.addEventListener('click', async (e) => {
                e.preventDefault();
                const id = deleteBtn.dataset.id;
                if (!id) return;

                if (!confirm("Sei sicuro di voler eliminare questa transazione?")) return;

                try {
                    const res = await fetch(`/transactions/${encodeURIComponent(id)}`, { method: 'DELETE' });
                    if (!res.ok) throw new Error('Errore eliminazione');
                    this.deleteCard(card);
                } catch (err) {
                    alert(err.message || err);
                }
            });
        }

        card.__listenersAttached = true;
    }
}
