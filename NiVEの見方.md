NiVE3では「オブジェクト」に相当する概念は**レイヤー(Layer)**、機能拡張は**エフェクト(Effect)**という名称で実装されています。

**エフェクト実装**
- インターフェース定義: `NiVE3.Plugin/Interfaces/IEffect.cs`, `IPropertyEditAwareEffect.cs`
- メタ情報: `NiVE3.Plugin/Attributes/EffectMetadataAttribute.cs`
- 各エフェクト本体(プリセット群): `NiVE3.PresetPlugin/Effect/` 配下、カテゴリ別ディレクトリ(`Blur`, `Channel`, `ColorCollection`, `Distortion`, `ExpressionControl`, `Generate`, `Keying`, `Noise`, `Simulation`, `Stylize`, `Transition` 等)に個別`.cs`ファイル
- エフェクト処理用ユーティリティ: `NiVE3.PresetPlugin/Effect/Util/`
- アプリ側のエフェクト管理(MVVM): `NiVE3/Model/EffectModel.cs`, `EffectModel_HistoryCommand.cs`, `EffectListModel.cs`, `EffectHandle.cs`, `NiVE3/ViewModel/EffectViewModel.cs`, `EffectListViewModel.cs`, `NiVE3/View/Part/EffectView.xaml.cs`, `EffectCollectionView.cs`, `NiVE3/View/Pane/EffectListView.xaml.cs`
- プロジェクトファイル保存形式: `NiVE3/Data/Json/Project/EffectData.cs`, プリセット保存: `NiVE3/Data/Json/Preset/EffectPreset.cs`
- Expression(数式制御)連携: `NiVE3/Expression/Wrapper/EffectWrapper.cs`
- OpenFX連携: `NiVE3.OpenFX/Integration/OfxEffectAdapter.cs`, `OfxEffectRegistry.cs`

**オブジェクト(レイヤー)実装**
- アプリ側モデル本体: `NiVE3/Model/LayerModel.cs`, `LayerModel_HistoryCommand.cs`
- ViewModel: `NiVE3/ViewModel/LayerViewModel.cs`, `LayerPropertyControllerViewModel.cs`
- View: `NiVE3/View/Part/LayerView.xaml.cs`, `LayerCollectionView.cs`, `LayerItemExpander.cs`, `MultipleStateLayerSwitch.cs`, `NiVE3/View/Pane/LayerPropertyControllerView.xaml.cs`
- 保存形式: `NiVE3/Data/Json/Project/LayerData.cs`
- Expression連携: `NiVE3/Expression/Wrapper/LayerWrapper.cs`, `LayerTransformPropertiesWrapper.cs`, `LayerTextPropertyWrapper.cs`, `LayerAudioLevelPropertiesWrapper.cs`, `LayerAudioLevelValuePropertyWrapper.cs`, `LayerOptionPropertiesWrapper.cs`
- プラグイン側の値オブジェクト: `NiVE3.Plugin/ValueObject/LayerInfo.cs`, `UseLayerImageTarget.cs`, `UseLayerAudioTarget.cs`