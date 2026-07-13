use std::collections::HashMap;
use balchug_common::atlas::{Atlas, AtlasItem, FontData, FontGlyph};

pub fn create_atlas() -> Atlas {
    let mut items = HashMap::with_capacity(10);
    items.insert(6, AtlasItem {id:6,x:1367,y:2,width:1361,height:1792,origin_width:1920,origin_height:2876});
    items.insert(3, AtlasItem {id:3,x:2,y:2,width:1361,height:1794,origin_width:1920,origin_height:2880});
    items.insert(2, AtlasItem {id:2,x:2732,y:3298,width:1361,height:795,origin_width:1920,origin_height:1280});
    items.insert(7, AtlasItem {id:7,x:2732,y:1600,width:1361,height:894,origin_width:1920,origin_height:1439});
    items.insert(9, AtlasItem {id:9,x:1367,y:3495,width:1361,height:517,origin_width:1920,origin_height:835});
    items.insert(8, AtlasItem {id:8,x:1367,y:1798,width:1361,height:894,origin_width:1920,origin_height:1438});
    items.insert(4, AtlasItem {id:4,x:2732,y:2499,width:1361,height:795,origin_width:1920,origin_height:1280});
    items.insert(5, AtlasItem {id:5,x:2,y:1800,width:1361,height:1794,origin_width:1920,origin_height:2880});
    items.insert(1, AtlasItem {id:1,x:1367,y:2696,width:1361,height:795,origin_width:1920,origin_height:1280});
    items.insert(0, AtlasItem {id:0,x:2732,y:2,width:1361,height:1594,origin_width:1920,origin_height:2560});
    Atlas {width:4096,height:4096,items}
}

pub fn create_font_atlas() -> Atlas {
    let mut items = HashMap::with_capacity(22);
    items.insert(8, AtlasItem {id:8,x:27,y:121,width:27,height:30,origin_width:27,origin_height:30});
    items.insert(16, AtlasItem {id:16,x:80,y:131,width:27,height:28,origin_width:27,origin_height:28});
    items.insert(12, AtlasItem {id:12,x:54,y:121,width:26,height:30,origin_width:26,origin_height:30});
    items.insert(3, AtlasItem {id:3,x:30,y:91,width:28,height:30,origin_width:28,origin_height:30});
    items.insert(5, AtlasItem {id:5,x:107,y:131,width:27,height:28,origin_width:27,origin_height:28});
    items.insert(6, AtlasItem {id:6,x:74,y:57,width:30,height:28,origin_width:30,origin_height:28});
    items.insert(11, AtlasItem {id:11,x:76,y:0,width:34,height:28,origin_width:34,origin_height:28});
    items.insert(18, AtlasItem {id:18,x:104,y:74,width:29,height:29,origin_width:29,origin_height:29});
    items.insert(17, AtlasItem {id:17,x:41,y:0,width:35,height:28,origin_width:35,origin_height:28});
    items.insert(13, AtlasItem {id:13,x:133,y:74,width:27,height:30,origin_width:27,origin_height:30});
    items.insert(2, AtlasItem {id:2,x:88,y:103,width:28,height:28,origin_width:28,origin_height:28});
    items.insert(10, AtlasItem {id:10,x:41,y:28,width:33,height:35,origin_width:33,origin_height:35});
    items.insert(4, AtlasItem {id:4,x:0,y:35,width:30,height:37,origin_width:30,origin_height:37});
    items.insert(1, AtlasItem {id:1,x:60,y:85,width:28,height:30,origin_width:28,origin_height:30});
    items.insert(9, AtlasItem {id:9,x:143,y:0,width:17,height:18,origin_width:17,origin_height:18});
    items.insert(7, AtlasItem {id:7,x:30,y:63,width:30,height:28,origin_width:30,origin_height:28});
    items.insert(15, AtlasItem {id:15,x:110,y:0,width:33,height:37,origin_width:33,origin_height:37});
    items.insert(14, AtlasItem {id:14,x:105,y:37,width:30,height:37,origin_width:30,origin_height:37});
    items.insert(0, AtlasItem {id:0,x:0,y:0,width:41,height:35,origin_width:41,origin_height:35});
    items.insert(19, AtlasItem {id:19,x:74,y:28,width:31,height:29,origin_width:31,origin_height:29});
    items.insert(20, AtlasItem {id:20,x:0,y:72,width:30,height:28,origin_width:30,origin_height:28});
    items.insert(21, AtlasItem {id:21,x:0,y:100,width:27,height:37,origin_width:27,origin_height:37});
    Atlas {width:256,height:256,items}
}

pub fn create_font() -> FontData {
    let mut glyphs = HashMap::with_capacity(22);
    glyphs.insert('й', FontGlyph {item_id:4,h_advance:20.1074,offset_x:-5.0000,offset_y:-31.0000});
    glyphs.insert('з', FontGlyph {item_id:8,h_advance:15.5536,offset_x:-6.0000,offset_y:-23.0000});
    glyphs.insert('л', FontGlyph {item_id:19,h_advance:19.0812,offset_x:-7.0000,offset_y:-22.0000});
    glyphs.insert('ч', FontGlyph {item_id:20,h_advance:18.7925,offset_x:-6.0000,offset_y:-22.0000});
    glyphs.insert('р', FontGlyph {item_id:14,h_advance:18.7605,offset_x:-6.0000,offset_y:-23.0000});
    glyphs.insert('Э', FontGlyph {item_id:15,h_advance:21.5986,offset_x:-6.0000,offset_y:-30.0000});
    glyphs.insert('ё', FontGlyph {item_id:21,h_advance:15.8742,offset_x:-5.0000,offset_y:-30.0000});
    glyphs.insert('а', FontGlyph {item_id:3,h_advance:16.5477,offset_x:-5.0000,offset_y:-23.0000});
    glyphs.insert('м', FontGlyph {item_id:11,h_advance:23.4907,offset_x:-5.0000,offset_y:-22.0000});
    glyphs.insert('г', FontGlyph {item_id:16,h_advance:14.4472,offset_x:-6.0000,offset_y:-22.0000});
    glyphs.insert('ы', FontGlyph {item_id:17,h_advance:24.8857,offset_x:-5.0000,offset_y:-22.0000});
    glyphs.insert('н', FontGlyph {item_id:6,h_advance:19.9951,offset_x:-5.0000,offset_y:-22.0000});
    glyphs.insert('о', FontGlyph {item_id:1,h_advance:17.7022,offset_x:-5.0000,offset_y:-23.0000});
    glyphs.insert('т', FontGlyph {item_id:2,h_advance:15.4573,offset_x:-6.0000,offset_y:-22.0000});
    glyphs.insert('М', FontGlyph {item_id:0,h_advance:30.4497,offset_x:-6.0000,offset_y:-29.0000});
    glyphs.insert('Т', FontGlyph {item_id:10,h_advance:20.3158,offset_x:-6.0000,offset_y:-29.0000});
    glyphs.insert('е', FontGlyph {item_id:13,h_advance:15.8742,offset_x:-5.0000,offset_y:-23.0000});
    glyphs.insert('с', FontGlyph {item_id:12,h_advance:14.9122,offset_x:-5.0000,offset_y:-23.0000});
    glyphs.insert('к', FontGlyph {item_id:18,h_advance:18.1031,offset_x:-5.0000,offset_y:-23.0000});
    glyphs.insert('и', FontGlyph {item_id:7,h_advance:20.1074,offset_x:-5.0000,offset_y:-22.0000});
    glyphs.insert('.', FontGlyph {item_id:9,h_advance:8.8511,offset_x:-4.0000,offset_y:-11.0000});
    glyphs.insert('в', FontGlyph {item_id:5,h_advance:17.0127,offset_x:-5.0000,offset_y:-22.0000});
    FontData {ascend:30.1130,height:37.3125,line_gap:0.0000,space_width:7.9211,glyphs}
}

pub fn get_letters() -> String {
    "рын тмйкгчисёеМолаЭ.Твз".to_string()
}
