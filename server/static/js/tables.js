export function initTable(selector, columns, data) {
    const element = document.querySelector(selector);
    if (!element) return null;

    const table = new Tabulator(selector, {
        data: data,
        columns: columns,
        layout: "fitColumns",
        pagination: "local",
        paginationSize: 10,
        movableColumns: true,
        resizableRows: true,
        placeholder: "No data is available",
        autoResize: true,
        height: "auto",
    });

    return table;
}

export async function deleteFromTable(path, confirmMessage, row) {
    if (!confirm(confirmMessage)) return;

    try {
        const response = await fetch(path, { method: 'DELETE' });
        if (!response.ok) throw new Error("Error deleting from table");

        row.delete();
    } catch (err) {
        alert(err);
    }
}