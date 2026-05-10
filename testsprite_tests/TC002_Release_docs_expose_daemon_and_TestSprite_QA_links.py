import asyncio
import os
from playwright import async_api


BASE_URL = os.environ.get("TESTSPRITE_BASE_URL") or os.environ.get("BASE_URL") or "https://bbrainfuckk.github.io/qorx/"


async def run_test():
    pw = None
    browser = None
    context = None

    try:
        pw = await async_api.async_playwright().start()
        browser = await pw.chromium.launch(
            headless=True,
            args=[
                "--window-size=1280,720",
                "--disable-dev-shm-usage",
                "--ipc=host",
                "--single-process",
            ],
        )
        context = await browser.new_context()
        context.set_default_timeout(10000)
        page = await context.new_page()

        await page.goto(BASE_URL, wait_until="networkidle", timeout=20000)

        server_link = page.locator("a[href$='SERVER.html']").first
        assert await server_link.count() == 1
        await server_link.click(timeout=10000)
        await page.wait_for_load_state("networkidle", timeout=20000)
        server_body = await page.locator("body").inner_text(timeout=10000)
        assert "official background runtime is the daemon" in server_body
        assert "qorx daemon start" in server_body
        assert "127.0.0.1:47187" in server_body

        await page.goto(BASE_URL, wait_until="networkidle", timeout=20000)
        testsprite_link = page.locator("a[href$='TESTSPRITE.html']").first
        assert await testsprite_link.count() == 1
        await testsprite_link.click(timeout=10000)
        await page.wait_for_load_state("networkidle", timeout=20000)
        qa_body = await page.locator("body").inner_text(timeout=10000)
        assert "TestSprite enterprise QA" in qa_body
        assert "TESTSPRITE_API_KEY" in qa_body
        assert "reachable" in qa_body
    finally:
        if context:
            await context.close()
        if browser:
            await browser.close()
        if pw:
            await pw.stop()


asyncio.run(run_test())
