export function initRuleModal(openId, modalId, closeId) {
    const openBtn = document.getElementById(openId);
    const modal = document.getElementById(modalId);
    const closeBtn = document.getElementById(closeId);

    openBtn.addEventListener("click", () => modal.classList.remove("hidden"));
    closeBtn.addEventListener("click", () => modal.classList.add("hidden"));
    window.addEventListener("click", e => { if (e.target === modal) modal.classList.add("hidden"); });
}

export async function fetchJson(url, options = {}) {
    const response = await fetch(url, options);

    if (!response.ok) {
        const text = await response.text();
        throw new Error(text || response.statusText);
    }

    const contentType = response.headers.get("content-type") || "";
    if (contentType.includes("application/json")) {
        return response.json();
    } else {
        return null;
    }
}

export function renderPreview(previewData, previewList, conflictSelections) {
    previewList.innerHTML = "";

    const template = document.getElementById("preview-item-template");

    previewData.forEach(txt => {
        const clone = template.content.cloneNode(true);

        clone.querySelector(".old-values .description").textContent = txt.description || "-";
        clone.querySelector(".old-values .label-old").textContent = txt.label_old_value || "-";
        clone.querySelector(".old-values .perc-old").textContent = txt.perc_to_exclude_old_value || "-";
        clone.querySelector(".old-values .category-old").textContent = txt.category_old_value || "-";

        clone.querySelector(".new-values .description").textContent = txt.description || "-";
        clone.querySelector(".new-values .label-new").textContent = txt.label_new_value || "-";
        clone.querySelector(".new-values .perc-new").textContent = txt.perc_to_exclude_new_value || "-";
        clone.querySelector(".new-values .category-new").textContent = txt.category_new_value || "-";

        const conflictsContainer = clone.querySelector(".conflicts");
        if (txt.conflicts && txt.conflicts.length > 1) {
            txt.conflicts.forEach(rule => {
                const div = document.createElement("div");
                div.innerHTML = `
                    <span>${rule.label} (${rule.category_id})</span>
                    <input type="radio" name="conflict_${txt.id}" data-tx-id="${txt.id}" data-rule-id="${rule.id}">
                `;
                conflictsContainer.appendChild(div);
            });
        }

        previewList.appendChild(clone);
    });

    previewList.querySelectorAll('input[type="radio"]').forEach(radio => {
        radio.addEventListener('change', e => {
            const txId = e.target.dataset.txId;
            const ruleId = e.target.dataset.ruleId;
            conflictSelections[txId] = ruleId;
        });
    });
}


export async function applyRules(accountId, conflictSelections, modal) {
    const conflictPayload = Object.entries(conflictSelections).map(([txId, ruleId]) => ({
        transaction_id: parseInt(txId),
        rule_id: parseInt(ruleId)
    }));

    try {
        if (conflictPayload.length > 0) {
            await fetchJson(`/accounts/${accountId}/rules/resolve_conflicts`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(conflictPayload)
            });
        }

        await fetchJson(`/accounts/${accountId}/rules/apply_rules`, {
            method: 'POST'
        });

        modal.classList.add("hidden");
        window.location.reload();
    } catch (err) {
        alert("Errore nel processo: " + err);
    }
}


export function initDragAndDrop(dropzones, rules, accountId) {
    rules.forEach(rule => {
        rule.addEventListener('dragstart', e => {
            e.dataTransfer.setData('text/plain', rule.dataset.ruleId);
        });
    });

    dropzones.forEach(zone => {
        zone.addEventListener('dragover', e => e.preventDefault());

        zone.addEventListener('drop', async e => {
            e.preventDefault();
            const ruleId = e.dataTransfer.getData('text/plain');
            const target = zone.id === 'active-rules' ? 'activate' : 'deactivate';
            const url = `/accounts/${accountId}/rules/${ruleId}/${target}`;

            try {
                const response = await fetch(url, { method: 'POST' });
                if (!response.ok) throw new Error(`${response.status} ${response.statusText}`);
                const ruleDiv = document.querySelector(`.rule[data-rule-id='${ruleId}']`);
                zone.appendChild(ruleDiv);
            } catch (err) {
                console.error('Errore nella richiesta POST:', err);
            }
        });
    });
}
