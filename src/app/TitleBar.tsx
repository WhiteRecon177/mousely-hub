import { getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";
import { RELEASES_URL } from "./links";

/** Own title bar (the Windows one is off): it takes the colours of the theme. */
export function TitleBar() {
  const win = () => getCurrentWindow();

  return (
    <div className="dd-titlebar" data-tauri-drag-region>
      <div className="dd-titlebar-name" data-tauri-drag-region>
        <svg className="dd-titlebar-logo" width="38" height="20" viewBox="14 118 484 276" fill="none" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
          <g stroke="currentColor" strokeWidth="14">
            <path d="M62 256H22M62 256L30 232M62 256L30 280" />
            <path d="M450 256H490M450 256L482 232M450 256L482 280" />
          </g>
          <g fill="currentColor" stroke="none">
            <circle cx="26" cy="232" r="13" /><circle cx="26" cy="280" r="13" />
            <circle cx="486" cy="232" r="13" /><circle cx="486" cy="280" r="13" />
          </g>
          <path d="M446 256 L445 273 L441 289 L436 303 L428 316 L419 327 L409 335 L399 341 L388 345 L377 347 L366 346 L356 345 L346 342 L336 337 L327 332 L318 326 L310 319 L303 312 L295 305 L288 297 L281 289 L275 281 L269 273 L262 264 L256 256 L250 248 L243 239 L237 231 L231 223 L224 215 L217 207 L209 200 L202 193 L194 186 L185 180 L176 175 L166 170 L156 167 L146 166 L135 165 L124 167 L113 171 L103 177 L93 185 L84 196 L76 209 L71 223 L67 239 L66 256 L67 273 L71 289 L76 303 L84 316 L93 327 L103 335 L113 341 L124 345 L135 347 L146 346 L156 345 L166 342 L176 337 L185 332 L194 326 L202 319 L209 312 L217 305 L224 297 L231 289 L237 281 L243 273 L250 264 L256 256 L262 248 L269 239 L275 231 L281 223 L288 215 L295 207 L303 200 L310 193 L318 186 L327 180 L336 175 L346 170 L356 167 L366 166 L377 165 L388 167 L399 171 L409 177 L419 185 L428 196 L436 209 L441 223 L445 239 L446 256" stroke="currentColor" strokeWidth="38" />
          <path d="M314 322 L305 315 L297 307 L290 299 L283 291 L276 282 L269 274 L263 265 L256 256 L249 247 L243 238 L236 230 L229 221 L222 213 L215 205 L207 197 L198 190" className="dd-titlebar-gap" strokeWidth="64" />
          <path d="M314 322 L305 315 L297 307 L290 299 L283 291 L276 282 L269 274 L263 265 L256 256 L249 247 L243 238 L236 230 L229 221 L222 213 L215 205 L207 197 L198 190" stroke="currentColor" strokeWidth="38" />
        </svg>
        <span data-tauri-drag-region>Mousely Hub</span>
      </div>
      <div className="dd-titlebar-btns">
        <button type="button" className="dd-tb-btn dd-tb-btn--gh" aria-label="GitHub" title="GitHub" onClick={() => void openUrl(RELEASES_URL)}>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
            <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8Z" />
          </svg>
        </button>
        <button type="button" className="dd-tb-btn" aria-label="Minimize" onClick={() => void win().minimize()}>
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true"><path d="M0 5.5h10" stroke="currentColor" strokeWidth="1" /></svg>
        </button>
        <button type="button" className="dd-tb-btn dd-tb-btn--close" aria-label="Close" onClick={() => void win().close()}>
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true"><path d="M0 0l10 10M10 0L0 10" stroke="currentColor" strokeWidth="1" /></svg>
        </button>
      </div>
    </div>
  );
}
