async function loadMessage() {
    const res = await fetch("/api/hello");
    const data = await res.json();
    document.getElementById("message").innerText = data.message;
}
