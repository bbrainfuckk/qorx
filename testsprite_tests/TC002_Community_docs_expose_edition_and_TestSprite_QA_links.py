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

        community_link = page.locator("a[href$='COMMUNITY.html']").first
        assert await community_link.count() == 1
        await community_link.click(timeout=10000)
        await page.wait_for_load_state("networkidle", timeout=20000)
        community_body = await page.locator("body").inner_text(timeout=10000)
        assert "Qorx Community Edition" in community_body
        assert "Qorx Ayie" in community_body
        assert "live public metrics" in community_body
        assert "cross-platform GitHub release assets" in community_body
        assert "Qorx Ayie Starter" in community_body
        assert "5,000 included Ayie/Cloud requests" in community_body
        assert "PyPI" in community_body
        assert "Arch/AUR" in community_body
        assert "daemon" in community_body
        assert "integrate" in community_body

        await page.goto(BASE_URL, wait_until="networkidle", timeout=20000)
        starter_link = page.locator("a[href$='AYIE_STARTER.html']").first
        assert await starter_link.count() == 1
        await starter_link.click(timeout=10000)
        await page.wait_for_load_state("networkidle", timeout=20000)
        starter_body = await page.locator("body").inner_text(timeout=10000)
        assert "Qorx Ayie Starter" in starter_body
        assert "5,000 included Ayie/Cloud requests" in starter_body
        assert "server-side" in starter_body

        await page.goto(BASE_URL, wait_until="networkidle", timeout=20000)
        science_link = page.locator("a[href$='SCIENCE_AND_MATH.html']").first
        assert await science_link.count() == 1
        await science_link.click(timeout=10000)
        await page.wait_for_load_state("networkidle", timeout=20000)
        science_body = await page.locator("body").inner_text(timeout=10000)
        assert "Baseline-to-Compact" in science_body
        assert "included_requests = 5000" in science_body
        assert "Reference papers and sources" in science_body

        await page.goto(BASE_URL, wait_until="networkidle", timeout=20000)
        testsprite_link = page.locator("a[href$='TESTSPRITE.html']").first
        assert await testsprite_link.count() == 1
        await testsprite_link.click(timeout=10000)
        await page.wait_for_load_state("networkidle", timeout=20000)
        qa_body = await page.locator("body").inner_text(timeout=10000)
        assert "TestSprite QA" in qa_body
        assert "TESTSPRITE_API_KEY" in qa_body
        assert "Community Edition" in qa_body
    finally:
        if context:
            await context.close()
        if browser:
            await browser.close()
        if pw:
            await pw.stop()


asyncio.run(run_test())
