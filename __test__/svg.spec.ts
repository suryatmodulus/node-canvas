import { join } from 'path'
import { readFileSync } from 'fs'

import test from 'ava'

import { convertSVGTextToPath, GlobalFonts } from '../index'

GlobalFonts.registerFromPath(join(__dirname, 'fonts', 'iosevka-slab-regular.ttf'))

const FIXTURE = readFileSync(join(__dirname, 'text.svg'), 'utf8')

test('convertSVGTextToPath should work', (t) => {
  const result = convertSVGTextToPath(FIXTURE)
  t.snapshot(result.toString('utf8'))
})
