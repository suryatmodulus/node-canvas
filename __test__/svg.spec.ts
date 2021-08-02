import { join } from 'path'

import test from 'ava'

import { convertSVGTextToPath, GlobalFonts } from '../index'

GlobalFonts.loadFontsFromDir(join(__dirname, 'fonts', 'iosevka-slab-regular.ttf'))

const fixture = `<svg height="1024px" width="2000px" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
  <path d="M32,87.344c0,327.678,265.635,593.313,593.313,593.313 M625.313,680.656C827.829,680.656,992,516.485,992,313.969 M992,313.969c0-125.162-101.464-226.625-226.625-226.625 M765.375,87.344c-77.354,0-140.063,62.708-140.063,140.062 M625.313,227.406c0,47.808,38.756,86.563,86.563,86.563 M711.876,313.969c29.546,0,53.499-23.953,53.499-53.5 M765.375,260.47c0-18.261-14.804-33.064-33.064-33.064 M732.311,227.406c-11.286,0-20.435,9.149-20.435,20.435" fill="none" id="id1" stroke-width="1"/>
  <text fill="#000" font-family="Iosevka Slab" font-size="29" font-weight="bold" id="id2" stroke="none">
    <textPath startOffset="100%" text-anchor="end" href="#id1">abc12345640123456723456789010111213141516171819202122232425262728293031323334353637383940123456789010111213141516171819202122232425262728293031323334353637383940xyz</textPath>
  </text>
</svg>
`

test('convertSVGTextToPath should work', (t) => {
  t.snapshot(convertSVGTextToPath(fixture).toString('utf8'))
})
