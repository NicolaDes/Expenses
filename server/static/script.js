
export async function deleteFromTable(path, confirmMessage, row) {
    if (!confirm(confirmMessage)) return;

    try {
        const response = await fetch(path, { method: 'DELETE' });
        if (!response.ok) throw new Error("Errore eliminando la transazione");

        row.remove();
    } catch (err) {
        alert(err);
    }
}