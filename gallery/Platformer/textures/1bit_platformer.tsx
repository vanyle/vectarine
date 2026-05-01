<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.10.2" name="tilemap" tilewidth="16" tileheight="16" spacing="1" tilecount="400" columns="20">
 <image source="1bit_platformer.png" width="339" height="339"/>
 <tile id="2" type="coin"/>
 <tile id="58" type="open_door"/>
 <tile id="59" type="exit_door"/>
 <tile id="76" type="npc"/>
 <tile id="77" type="npc2"/>
 <tile id="225" type="barrel"/>
 <tile id="241" type="player"/>
 <tile id="248" type="torch"/>
 <wangsets>
  <wangset name="base" type="edge" tile="-1">
   <wangcolor name="dirt" color="#ff0000" tile="-1" probability="1"/>
   <wangtile tileid="87" wangid="0,0,1,0,1,0,0,0"/>
   <wangtile tileid="88" wangid="0,0,1,0,1,0,1,0"/>
   <wangtile tileid="89" wangid="0,0,0,0,1,0,1,0"/>
   <wangtile tileid="107" wangid="1,0,1,0,1,0,0,0"/>
   <wangtile tileid="108" wangid="1,0,1,0,1,0,1,0"/>
   <wangtile tileid="109" wangid="1,0,0,0,1,0,1,0"/>
   <wangtile tileid="127" wangid="1,0,1,0,0,0,0,0"/>
   <wangtile tileid="128" wangid="1,0,1,0,0,0,1,0"/>
   <wangtile tileid="129" wangid="1,0,0,0,0,0,1,0"/>
  </wangset>
  <wangset name="base2" type="corner" tile="-1">
   <wangcolor name="" color="#ff0000" tile="-1" probability="1"/>
   <wangtile tileid="87" wangid="0,0,0,1,0,0,0,0"/>
   <wangtile tileid="88" wangid="0,0,0,1,0,1,0,0"/>
   <wangtile tileid="89" wangid="0,0,0,0,0,1,0,0"/>
   <wangtile tileid="107" wangid="0,1,0,1,0,0,0,0"/>
   <wangtile tileid="108" wangid="0,1,0,1,0,1,0,1"/>
   <wangtile tileid="109" wangid="0,0,0,0,0,1,0,1"/>
   <wangtile tileid="127" wangid="0,1,0,0,0,0,0,0"/>
   <wangtile tileid="128" wangid="0,1,0,0,0,0,0,1"/>
   <wangtile tileid="129" wangid="0,0,0,0,0,0,0,1"/>
  </wangset>
 </wangsets>
</tileset>
